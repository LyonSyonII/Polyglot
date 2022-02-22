use heck::CamelCase;
use pest_meta::ast::{Expr, Rule};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use std::collections::{HashMap, HashSet};

/// Defines how one rule (Bar) is used inside another (Foo).
#[derive(Copy, Clone)]
struct RuleRefInfo {
    /// Each parse tree for Foo contains at least `low` Bar instances.
    low: usize,
    /// Each parse tree for Foo containse at most `high` Bar instances.
    /// None means "infinity"
    high: Option<usize>,
}

impl RuleRefInfo {
    fn merge(self, other: RuleRefInfo) -> RuleRefInfo {
        RuleRefInfo {
            low: self.low.saturating_add(other.low),
            high: match (self.high, other.high) {
                (Some(h1), Some(h2)) => h1.checked_add(h2),
                _ => None,
            },
        }
    }

    fn getters(self) -> Getters {
        let mut g = Getters {
            iterator: false,
            exact: false,
            opt: false,
            head: 0,
        };
        // this getter is always applicable
        g.iterator = true;
        if self.low == 1 && self.high == Some(1) {
            g.exact = true;
        }
        if self.low == 0 && self.high == Some(1) {
            g.opt = true;
        }
        g.head = std::cmp::min(HEAD_MAX, self.low);
        g
    }
}

/// Represents what getter functions can be derived
struct Getters {
    /// returns Iterator<Child>
    iterator: bool,
    /// returns Child
    exact: bool,
    /// returns Option<Child>
    opt: bool,
    /// example: if head = 3, then we have:
    /// get_first: fn -> Child;
    /// get_second: fn -> Child;
    /// get_third: fn -> Child;
    head: usize,
}

const HEAD_MAX: usize = 4;

impl Default for RuleRefInfo {
    fn default() -> Self {
        RuleRefInfo {
            low: 0,
            high: Some(0),
        }
    }
}

fn combine_maps<F: Fn(RuleRefInfo, RuleRefInfo) -> RuleRefInfo>(
    lhs: HashMap<String, RuleRefInfo>,
    rhs: HashMap<String, RuleRefInfo>,
    f: F,
) -> HashMap<String, RuleRefInfo> {
    let mut out = HashMap::new();
    for (name, left_val) in &lhs {
        let right_val = rhs.get(name).copied().unwrap_or_default();
        out.insert(name.clone(), f(*left_val, right_val));
    }
    for (name, right_val) in &rhs {
        if lhs.contains_key(name) {
            continue;
        }
        out.insert(name.clone(), f(RuleRefInfo::default(), *right_val));
    }
    out
}

/// takes an expr, representing rule, and estimates which other rules can be
/// direct children
fn rule_idents(expr: &Expr) -> HashMap<String, RuleRefInfo> {
    match expr {
        Expr::Ident(ident) => {
            let mut m = HashMap::new();
            m.insert(
                ident.clone(),
                RuleRefInfo {
                    low: 1,
                    high: Some(1),
                },
            );
            m
        }
        // these expressions don't produce nodes
        Expr::Str(_)
        | Expr::Insens(_)
        | Expr::Range(_, _)
        | Expr::PeekSlice(_, _)
        | Expr::PosPred(_)
        | Expr::NegPred(_)
        | Expr::Skip(_) => HashMap::new(),
        Expr::Seq(lhs, rhs) => {
            let lhs = rule_idents(lhs);
            let rhs = rule_idents(rhs);
            combine_maps(lhs, rhs, |lhs, rhs| lhs.merge(rhs))
        }
        Expr::Choice(lhs, rhs) => {
            let lhs = rule_idents(lhs);
            let rhs = rule_idents(rhs);
            combine_maps(lhs, rhs, |lhs, rhs| RuleRefInfo {
                low: std::cmp::min(lhs.low, rhs.low),
                high: match (lhs.high, rhs.high) {
                    (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
                    (a, b) => a.or(b),
                },
            })
        }
        Expr::Opt(inner) => {
            let mut map = rule_idents(inner);
            // now we are uncertain about low bounds.
            for info in map.values_mut() {
                info.low = 0;
            }
            map
        }
        Expr::Rep(inner) => {
            let mut map = rule_idents(inner);
            // now we are uncertain about all bounds.
            for info in map.values_mut() {
                info.low = 0;
                info.high = None;
            }
            map
        }
        Expr::RepOnce(inner) => {
            let mut map = rule_idents(inner);
            // now we are uncertain about high bounds.
            for info in map.values_mut() {
                info.high = None;
            }
            map
        }
        Expr::RepExact(inner, cnt) => {
            let cnt = *cnt as usize;
            let mut map = rule_idents(inner);
            // in fact we just multiply bounds by cnt
            for info in map.values_mut() {
                info.low = info.low.saturating_mul(cnt);
                info.high = info.high.and_then(|high| high.checked_mul(cnt));
            }
            map
        }
        Expr::RepMin(inner, cnt) => {
            let cnt = *cnt as usize;
            let mut map = rule_idents(inner);
            for info in map.values_mut() {
                info.low = info.low.saturating_mul(cnt);
                info.high = None;
            }
            map
        }
        Expr::RepMax(inner, cnt) => {
            let cnt = *cnt as usize;
            let mut map = rule_idents(inner);
            for info in map.values_mut() {
                info.low = 0;
                info.high = info.high.and_then(|high| high.checked_mul(cnt));
            }
            map
        }
        Expr::RepMinMax(inner, cnt_min, cnt_max) => {
            let cnt_min = *cnt_min as usize;
            let cnt_max = *cnt_max as usize;
            let mut map = rule_idents(inner);
            for info in map.values_mut() {
                info.low = info.low.saturating_mul(cnt_min);
                info.high = info.high.and_then(|high| high.checked_mul(cnt_max))
            }
            map
        }
        Expr::Push(inner) => rule_idents(inner),
    }
}

fn generate_module_header() -> TokenStream {
    quote! {
        use super::Rule;
        use pest::iterators::{Pair, Pairs};

        fn report_rule_mismatch(expected: Rule, actual: Rule) {
            panic!("expected {:?}, got {:?}", expected, actual);
        }
    }
}

fn is_rule_special(name: &str) -> bool {
    name.chars().all(|c| !c.is_ascii_lowercase())
}

/// Checks that rule is defined as {child1 | child 2 | child 3}
fn is_disjoint_choice(rule: &Rule) -> (bool, HashSet<String>) {
    let mut used = HashSet::new();
    let ok = is_disjoint_choice_inner(&rule.expr, &mut used);
    (ok, used)
}

fn is_disjoint_choice_inner(expr: &Expr, used: &mut HashSet<String>) -> bool {
    let (lhs, rhs) = match expr {
        Expr::Choice(lhs, rhs) => (lhs, rhs),
        _ => return false,
    };
    let rhs = match &**rhs {
        Expr::Ident(ident) => ident,
        _ => return false,
    };
    used.insert(rhs.clone());
    match &**lhs {
        Expr::Ident(ident) => used.insert(ident.clone()),
        other => is_disjoint_choice_inner(other, used),
    }
}

fn make_ident(s: &str) -> syn::Ident {
    syn::Ident::new(s, Span::call_site())
}

fn generate_rule_all_children_iter(rule: &Rule) -> Option<TokenStream> {
    let (ok, children) = is_disjoint_choice(rule);
    if !ok {
        return None;
    }
    let mut out = TokenStream::new();

    let enum_name = make_ident(&format!("{}Children", rule.name.to_camel_case()));
    {
        // at first, we will generate enum
        let mut variants = TokenStream::new();
        for child in &children {
            let struct_name = child.to_camel_case();
            let struct_name_ident = make_ident(&struct_name);
            let variant = quote! {
                #struct_name_ident(#struct_name_ident<'i>),
            };
            variants.extend(variant);
        }
        let enum_body: TokenStream = proc_macro2::TokenTree::Group(proc_macro2::Group::new(
            proc_macro2::Delimiter::Brace,
            variants,
        ))
        .into();
        let enum_header = quote! {
            pub enum #enum_name<'i>
        };
        out.extend(enum_header);
        out.extend(enum_body);
    }
    {
        // now we generate function that returns that enum
        let rule_struct_name = make_ident(&rule.name.to_camel_case());
        let mut fn_body = TokenStream::new();
        for child in &children {
            let child_struct_name = make_ident(&child.to_camel_case());
            let child_arm = quote! {
                if pair.as_rule() == #child_struct_name::RULE {
                    return #enum_name::#child_struct_name(#child_struct_name::new(pair));
                }
            };
            fn_body.extend(child_arm);
        }
        let fn_header = quote! {
            impl<'i> #rule_struct_name<'i> {
                pub fn to_enum (&self) -> #enum_name {
                    let mut pair = self.pairs().next().expect("pest-typed-tree bug");
                    #fn_body
                    panic!("pest-typed-tree bug: this should be unreachable")
                }
            }
        };
        out.extend(fn_header);
    }
    Some(out)
}

fn generate_rule_struct(rule: Rule) -> TokenStream {
    let rule_name = &rule.name;
    let rule_name_ident = make_ident(&rule.name);
    let struct_name = rule.name.to_camel_case();
    let struct_name = make_ident(&struct_name);
    let struct_def = quote! {
          pub struct #struct_name<'i>(Pair<'i, Rule>);

          impl<'i> std::fmt::Debug for #struct_name<'i> {
              fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    let mut helper = f.debug_tuple(#rule_name);
                    helper.field(&self.0);
                    helper.finish()
              }
          }

          impl<'i> std::clone::Clone for #struct_name<'i> {
              fn clone(&self) -> Self {
                  Self(self.0.clone())
              }
          }
    };
    let mut out = struct_def;

    let common_fns = quote! {
        impl<'i> #struct_name<'i> {
            pub const RULE: Rule = Rule::#rule_name_ident;

            pub fn new(p: Pair<'i, Rule>) -> #struct_name {
                if p.as_rule() != Self::RULE {
                    report_rule_mismatch(Self::RULE, p.as_rule());
                }
                #struct_name(p)
            }

            pub fn text(&self) -> &str {
                self.0.as_str()
            }

            pub fn span(&self) -> pest::Span {
                self.0.as_span()
            }

            pub fn pairs(&self) -> Pairs<'i, Rule> {
               self.0.clone().into_inner()
            }
        }
    };

    out.extend(common_fns);

    if let Some(tokens) = generate_rule_all_children_iter(&rule) {
        out.extend(tokens);
    }

    let children = rule_idents(&rule.expr);

    for (child_rule, ref_info) in children {
        if is_rule_special(&child_rule) {
            continue;
        }
        let getters = ref_info.getters();
        let child_rule_struct_name = make_ident(&child_rule.to_camel_case());
        if getters.exact {
            let ident = make_ident(&format!("get_{}", &child_rule));
            let code = quote! {
                impl<'i> #struct_name<'i> {
                    pub fn #ident (&self) -> #child_rule_struct_name {
                        self
                        .pairs()
                        .find(|pair| pair.as_rule() == #child_rule_struct_name::RULE)
                        .map(|pair| #child_rule_struct_name::new(pair))
                        .expect("bug in pest-typed-tree: child of requested type missing")
                    }
                }
            };
            out.extend(code);
        }
        if getters.iterator {
            let ident = make_ident(&format!("list_{}", &child_rule));
            let code = quote! {
                impl<'i> #struct_name<'i> {
                    pub fn #ident (&self) -> impl Iterator<Item = #child_rule_struct_name> {
                        self
                        .pairs()
                        .filter(|pair| pair.as_rule() == #child_rule_struct_name::RULE)
                        .map(|pair| #child_rule_struct_name::new(pair))
                    }
                }
            };
            out.extend(code);
        }
        if getters.opt {
            // we use same identifier for `exact` but it is safe
            // because they never overlap
            let ident = make_ident(&format!("get_{}", &child_rule));
            let code = quote! {
                impl<'i> #struct_name<'i> {
                    pub fn #ident (&self) -> Option<#child_rule_struct_name> {
                        self
                        .pairs()
                        .find(|pair| pair.as_rule() == #child_rule_struct_name::RULE)
                        .map(|pair| #child_rule_struct_name::new(pair))
                    }
                }
            };
            out.extend(code);
        }
        for num in 0..HEAD_MAX {
            if num == getters.head {
                break;
            }
            let word = ["first", "second", "third"][num];
            let ident = make_ident(&format!("get_{}_{}", word, &child_rule));
            let code = quote! {
                impl<'i> #struct_name<'i> {
                pub fn #ident (&self) -> #child_rule_struct_name {
                    self
                    .pairs()
                    .filter(|pair| pair.as_rule() == #child_rule_struct_name::RULE)
                    .map(|pair| #child_rule_struct_name::new(pair))
                    .nth(#num)
                    .expect("bug in pest-typed-tree")
                }
                }
            };
            out.extend(code);
        }
    }

    out
}

pub fn generate(grammar: String) -> proc_macro::TokenStream {
    let pest_ast = pest_meta::parser::parse(pest_meta::parser::Rule::grammar_rules, &grammar)
        .expect("failed to parse pest grammar");
    let pest_ast = pest_meta::parser::consume_rules(pest_ast).expect("failed to build grammar ast");
    let mut module = TokenStream::new();
    module.extend(generate_module_header());
    for rule in pest_ast {
        module.extend(generate_rule_struct(rule));
    }
    // now let's wrap it into module
    let mut out: TokenStream = quote! {
        mod nodes
    }
    .into();

    let braced_def = TokenTree::Group(proc_macro2::Group::new(
        proc_macro2::Delimiter::Brace,
        module,
    ));
    out.extend(std::iter::once(braced_def));
    out.into()
}

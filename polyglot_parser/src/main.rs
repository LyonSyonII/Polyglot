use clap::Parser as P;
use inner::inner;
use pest::iterators::{Pair, Pairs};
use pest::Parser as Pest;
use serde::{Deserialize, Serialize, Serializer};

mod tree;
use show_my_errors::{AnnotationList, Stylesheet};
use tree::*;

// TODO! Check all values on list and dictionary too see if all have the same type

#[derive(clap::Parser)]
#[clap(version, about)]
struct Cli {
    file: std::path::PathBuf,
    #[clap(short, long)]
    debug: bool
}

#[derive(pest_derive::Parser, pest_typed_tree::TypedTree, Serialize, Deserialize, Debug)]
#[grammar = "grammar.pest"]
struct Parser;

fn main() {
    parse();
}

fn parse() {
    let cli = Cli::parse();
    let mut global = Scope::new(vec![]);
    global.set_file(cli.file.clone(), std::fs::read_to_string(&cli.file).unwrap());
    
    let file = global.file().to_owned();
    let root = nodes::Main::new(Parser::parse(Rule::Main, &file).unwrap().next().unwrap());
    
    let mut main = Main::new(Vec::new());
    for expr in root.list_Expr() {
        global.change_ctx(expr.text().into());
        main.0.push(parse_expr(expr, &mut global));
    }
    
    if global.err_found() && !cli.debug {
        return;
    }

    let buffer = serde_yaml::to_string(&main).unwrap();
    if cli.debug { println!("{buffer}") }
    let path = global.file_name().with_extension("yml");
    std::fs::write(path, buffer).unwrap();
}

fn parse_expr(expr: nodes::Expr, scope: &mut Scope) -> Expr {
    match expr.to_enum() {
        nodes::ExprChildren::Init(init) => Expr::Init(Box::new(parse_init(init, scope))),
        nodes::ExprChildren::Decl(_) => todo!(),
        nodes::ExprChildren::Typedef(_) => todo!(),
        nodes::ExprChildren::Assig(_) => todo!(),
        nodes::ExprChildren::ListRemAssig(_) => todo!(),
        nodes::ExprChildren::AddAssig(_) => todo!(),
        nodes::ExprChildren::SubAssig(_) => todo!(),
        nodes::ExprChildren::MulAssig(_) => todo!(),
        nodes::ExprChildren::DivAssig(_) => todo!(),
        nodes::ExprChildren::ModAssig(_) => todo!(),
        nodes::ExprChildren::PowAssig(_) => todo!(),
    }
}

fn parse_init(init: nodes::Init, scope: &mut Scope) -> Init {
    let name = init.get_Name().text().to_owned();
    let node_v = init.get_Value();
    let parsed_v = parse_value(&node_v, scope);
    
    let ty = match init.get_Type() {
        // Check if variable is of the same type as assignment
        Some(node_t) => {
            let t = parse_type(&node_t);
            let vt = parse_type_from_value(&parsed_v, scope);
            if t == vt {
                t
            } else {
                printerr(&node_v.range(), "wrong assignment type", &format!("expected '{t}', found '{vt}'"), scope);
                Type::Err
            }
        }
        // No explicit type annotation, inferring type from value
        None => parse_type_from_value(&parsed_v, scope),
    };

    scope.insert(name.clone(), ty.clone());
    Init::new(name, ty, parsed_v, scope.ctx().into())
}

fn parse_value(value: &nodes::Value, scope: &mut Scope) -> Value {
    match value.to_enum() {
        nodes::ValueChildren::Int(i) => Value::Int(i.text().parse().unwrap()),
        nodes::ValueChildren::Num(n) => Value::Num(n.text().parse().unwrap()),
        nodes::ValueChildren::Bool(b) => match b.text() {
            "true" => Value::Bool(Bool::Primitive(true)),
            "false" => Value::Bool(Bool::Primitive(false)),
            _ => Value::Bool(Bool::Cmp(parse_cmp(b.list_Cmp().next().unwrap()))),
        },
        nodes::ValueChildren::Char(c) => Value::Char(c.text().as_bytes()[0] as char),
        nodes::ValueChildren::Str(s) => Value::Str(
            s.text()
                .strip_prefix('"')
                .unwrap()
                .strip_suffix('"')
                .unwrap()
                .into(),
        ),
        nodes::ValueChildren::Tuple(t) => Value::Tuple(
            t.list_Value()
                .map(|val| parse_value(&val, scope))
                .collect::<Vec<Value>>(),
        ),
        nodes::ValueChildren::Struct(s) => Value::Struct ({
            s.list_StructVal().map(|sval| {(
                sval.get_Name().text().to_owned(),
                parse_value(&sval.get_Value(), scope)
            )}).collect()
        }),
        nodes::ValueChildren::TupleAccess(ta) => Value::TupleAccess {
            name: ta.get_Name().text().into(),
            access_mode: match ta.get_TupleAccessType().to_enum() {
                nodes::TupleAccessTypeChildren::Name(n) => TupleAccessMode::Member(n.text().into()),
                nodes::TupleAccessTypeChildren::Index(i) => {
                    TupleAccessMode::Index(i.text().parse().unwrap())
                }
            },
            name_range: ta.get_Name().range(),
            access_range: ta.get_TupleAccessType().range(),
        },
        nodes::ValueChildren::List(l) => Value::List(
            l.list_Value()
                .map(|val| parse_value(&val, scope))
                .collect::<Vec<Value>>(),
        ),
        // TODO! Check if list is a List or a Dict and access it properly
        nodes::ValueChildren::ListAccess(la) => {
            let name = la.get_Name().text().into();
            let name_range = la.get_Name().range();
            let access_range = {
                let range = la.get_Value().range();
                range.start-1..range.end+1
            };
            let list_type = if let Some(t) = scope.get(&name).cloned() {
                t
            } else {
                return printerr(&name_range, "accessed invalid list/dictionary", "list/dictionary does not exist", scope).value_err()
            };
            
            let (access_type, access_mode) = match list_type {
                Type::List(list) => {
                    (*list,
                        match la.get_Value().to_enum() {
                        nodes::ValueChildren::Int(i) => ListAccessMode::List(
                            if let Ok(i) = i.text().parse() {
                                i
                            } else {
                                return printerr(&access_range, "negative index", "index is negative, lists can only be accessed with positive numbers", scope).value_err()
                            }),
                        _ => return printerr(&access_range, "accessing list as a dictionary", format!("use the index of the element you want to access instead: {name}[0]"), scope).value_err()
                    })
                },
                Type::Dict(dict) => {
                    let value = parse_value(&la.get_Value(), scope);
                    let value_type = parse_type_from_value(&value, scope);
                    if dict.0 != value_type {
                        return printerr(&access_range, "wrong access type", format!("expected {} found {}", dict.0, value_type), scope).value_err()
                    } else {
                        (dict.1, ListAccessMode::Dict(Box::new(value)))
                    }
                },
                Type::Custom(_) => todo!(),
                _ => return printerr(&name_range, "variable exists but is not a list/dictionary", "not a list/dictionary", scope).value_err()
            };
           
            Value::ListAccess {
                name,
                access_type,
                access_mode,
                name_range,
                access_range
            }
        },
        nodes::ValueChildren::Dict(d) => Value::Dict(
            d.list_DictPair()
                .map(|pair| {
                    (
                        parse_value(&pair.get_first_Value(), scope),
                        parse_value(&pair.get_second_Value(), scope),
                    )
                })
                .collect::<Vec<(Value, Value)>>(),
        ),
        nodes::ValueChildren::Name(n) => Value::Var { name: n.text().into(), range: n.range() },

        // TODO! Complex values

        // TODO! Check if both values are of the same type
        nodes::ValueChildren::Op(op) => Value::Op(),
        nodes::ValueChildren::RetExpr(_) => todo!(),
        nodes::ValueChildren::Call(_) => todo!(),
    }
}

fn parse_type(ty: &nodes::Type) -> Type {
    match ty.to_enum() {
        nodes::TypeChildren::TInt(_) => Type::Int,
        nodes::TypeChildren::TNum(_) => Type::Num,
        nodes::TypeChildren::TBool(_) => Type::Bool,
        nodes::TypeChildren::TChar(_) => Type::Char,
        nodes::TypeChildren::TStr(_) => Type::Str,
        nodes::TypeChildren::TTuple(t) => Type::Tuple(
            t.list_Type().map(|t| parse_type(&t)).collect::<Vec<Type>>(),
        ),
        nodes::TypeChildren::TStruct(s) => Type::Struct(
            s.list_StructMem().map(|mem| (
                mem.get_Name().text().into(), parse_type(&mem.get_Type())
            )).collect()
        ),
        nodes::TypeChildren::TList(l) => Type::List(Box::new(parse_type(&l.get_Type()))),

        nodes::TypeChildren::TDict(d) => Type::Dict(Box::new((
            parse_type(&d.get_first_Type()),
            parse_type(&d.get_second_Type()),
        ))),

        // On typedef add type to variables, then check its value and return it
        nodes::TypeChildren::TCustom(c) => Type::Custom(c.text().into()),
    }
}

fn parse_type_from_value(value: &Value, scope: &mut Scope) -> Type {
    match value {
        Value::Int(_) => Type::Int,
        Value::Num(_) => Type::Num,
        Value::Bool(_) => Type::Bool,
        Value::Char(_) => Type::Char,
        Value::Str(_) => Type::Str,
        Value::Tuple(t) => Type::Tuple(
            t
                .iter()
                .map(|val| parse_type_from_value(val, scope))
                .collect::<Vec<Type>>(),
        ),
        Value::Struct(members) => Type::Struct(
            members
                .iter()
                .map(|mem| (mem.0.clone(), parse_type_from_value(&mem.1, scope)))
                .collect()
        ),
        Value::List(l) => Type::List(Box::new(parse_type_from_value(&l[0], scope))),
        Value::Dict(d) => Type::Dict(Box::new((
            parse_type_from_value(&d[0].0, scope),
            parse_type_from_value(&d[0].1, scope),
        ))),
        Value::Var {name, range} => {
            if let Some(var_t) = scope.get(name).cloned() {
                var_t
            } else {
                printerr(range, &format!("variable '{name}' does not exist"), "not declared", scope).type_err()
            }
        }
        Value::TupleAccess { name, access_mode: access_type, name_range, access_range} => {
            let tuple_type = if let Some(ty) = scope.get_mut(name){
                ty
            } else {
                return printerr(name_range, "accessed invalid tuple/struct", "struct does not exist", scope).type_err()
            };
            
            match access_type {
                TupleAccessMode::Member(member) => {
                    let struct_t = inner!(tuple_type, if Type::Struct, else { return printerr(access_range, "accessed tuple by member name", format!("use index instead: {name}.0"), scope).type_err() });
                    struct_t.sort_unstable();
                    if let Ok(ty) = struct_t.binary_search_by_key(&member, |(a, _)| a) {
                        struct_t[ty].1.clone()
                    } else {
                        printerr(access_range, format!("member '{name}.{member}' does not exist"), "not declared", scope);
                        Type::Err
                    }
                }
                TupleAccessMode::Index(index) => {
                    let tuple_t = inner!(tuple_type, if Type::Tuple, else { return printerr(access_range, "accessed struct by index", format!("use member name instead: {name}.member"), scope).type_err()});
                    if let Some(ty) = tuple_t.get(*index) {
                        ty.clone()
                    } else {
                        let n_elems = tuple_t.len();
                        let n_elems = if n_elems > 1 {
                            format!("{} elements", n_elems)
                        } else {
                            "1 element".into()
                        };
                        
                        printerr(&(name_range.start..access_range.end), "index out of bounds", format!("tuple has {}, trying to access element number {}", n_elems, index+1), scope).type_err()
                    }
                }
            }
        }
        Value::ListAccess { access_type, ..} => {
            access_type.clone()
        }
        // TODO! Complex values
        Value::Op(_) => todo!(),
        Value::Call{ name, args} => todo!(),
        Value::RetExpr(_) => todo!(),
        Value::Err => Type::Err
    }
}

fn parse_cmp(cmp: nodes::Cmp) -> Cmp {
    todo!()
}


struct PrintErr;
impl PrintErr {
    pub fn type_err(self) -> Type { Type::Err }
    
    pub fn value_err(self) -> Value { Value::Err }
}

fn printerr(range: &std::ops::Range<usize>, header: impl AsRef<str>, text: impl AsRef<str>, scope: &mut Scope) -> PrintErr {
    let mut list = AnnotationList::new(scope.file_name().to_string_lossy(), scope.file());
    list.error(range.start..range.end, header.as_ref(), text.as_ref()).unwrap();
    list.show_stderr(&Stylesheet::colored()).unwrap();
    println!();
    scope.err();
    PrintErr
}

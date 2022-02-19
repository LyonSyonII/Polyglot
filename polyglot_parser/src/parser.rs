#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::tree::*;
use clap::Parser as P;
use either::Either;
use inner::inner;
use pest::iterators::{Pair, Pairs};
use pest::Parser as Pest;
use serde::{Deserialize, Serialize, Serializer};
use show_my_errors::{AnnotationList, Stylesheet};

#[derive(pest_derive::Parser, pest_typed_tree::TypedTree, Serialize, Deserialize, Debug)]
#[grammar = "grammar.pest"]
pub struct Parser;

pub fn parse(file: &std::path::Path, debug: bool) -> Result<Main, ParseErr> {
    let mut global = Scope::new(vec![], HashMap::new());
    global.set_file(file.into(), std::fs::read_to_string(&file).unwrap());

    let file = global.file_as_str().to_owned();
    let root = nodes::Main::new(Parser::parse(Rule::Main, &file).unwrap().next().unwrap());
    let mut err_found = false;

    let mut main = Main::new(Vec::new());
    for expr in root.list_Expr() {
        let expr = parse_expr(expr, &mut global);
        err_found = err_found || expr == Expr::Err;
        main.0.push(expr)
    }

    if err_found && !debug {
        Err(ParseErr)
    } else {
        Ok(main)
    }
}

fn parse_expr(expr: nodes::Expr, scope: &mut Scope) -> Expr {
    match expr.to_enum() {
        nodes::ExprChildren::Use(u) => todo!(),
        nodes::ExprChildren::ModuleAccess(m) => todo!(),
        nodes::ExprChildren::Init(init) => parse_init(init, scope),
        nodes::ExprChildren::Decl(decl) => parse_decl(decl, scope),
        nodes::ExprChildren::Assig(assig) => parse_assig(assig, scope),
        nodes::ExprChildren::ListRemAssig(listrem) => parse_list_remove_assign(listrem, scope),
        nodes::ExprChildren::AddAssig(aa) => parse_assig_op(
            aa.get_Name(),
            aa.get_Value(),
            Op::Add(Box::new((Value::Err, Value::Err))),
            aa.text().into(),
            scope,
        ),
        nodes::ExprChildren::SubAssig(sa) => parse_assig_op(
            sa.get_Name(),
            sa.get_Value(),
            Op::Sub(Box::new((Value::Err, Value::Err))),
            sa.text().into(),
            scope,
        ),
        nodes::ExprChildren::MulAssig(ma) => parse_assig_op(
            ma.get_Name(),
            ma.get_Value(),
            Op::Mul(Box::new((Value::Err, Value::Err))),
            ma.text().into(),
            scope,
        ),
        nodes::ExprChildren::DivAssig(da) => parse_assig_op(
            da.get_Name(),
            da.get_Value(),
            Op::Div(Box::new((Value::Err, Value::Err))),
            da.text().into(),
            scope,
        ),
        nodes::ExprChildren::ModAssig(ma) => parse_assig_op(
            ma.get_Name(),
            ma.get_Value(),
            Op::Mod(Box::new((Value::Err, Value::Err))),
            ma.text().into(),
            scope,
        ),
        nodes::ExprChildren::PowAssig(pa) => parse_assig_op(
            pa.get_Name(),
            pa.get_Value(),
            Op::Pow(Box::new((Value::Err, Value::Err))),
            pa.text().into(),
            scope,
        ),
        nodes::ExprChildren::Typedef(t) => {
            let name = t.get_Name().to_string();
            let r#type = parse_type(&t.get_Type(), scope);
            scope.insert(name.clone(), r#type.clone());
            Expr::Typedef { name, r#type }
        }
        nodes::ExprChildren::If(_) => todo!(),
        nodes::ExprChildren::Fn(f) => parse_fn(f, scope),
        nodes::ExprChildren::Call(c) => parse_call(c, scope),
    }
}

fn parse_init(init: nodes::Init, scope: &mut Scope) -> Expr {
    let name = init.get_Name().to_string();
    let node_v = init.get_Value();
    let parsed_v = parse_value(&node_v, scope);
    if parsed_v == Value::Err {
        return Expr::Err;
    }

    let ty = match init.get_Type() {
        // Check if variable is of the same type as assignment
        Some(node_t) => {
            let t = parse_type(&node_t, scope);
            let vt = parse_type_from_value(&parsed_v, scope);
            if t == vt {
                t
            } else {
                return printerr(
                    &node_v.range(),
                    "wrong assignment type",
                    &format!("expected '{t}', found '{vt}'"),
                    scope,
                )
                .expr_err();
            }
        }
        // No explicit type annotation, inferring type from value
        None => parse_type_from_value(&parsed_v, scope),
    };

    scope.insert(name.clone(), ty.clone());
    Expr::Init {
        name,
        r#type: ty,
        value: parsed_v,
        context: init.text().to_owned(),
    }
}

fn parse_decl(decl: nodes::Decl, scope: &mut Scope) -> Expr {
    let name = decl.get_Name().to_string();
    let r#type = parse_type(&decl.get_Type(), scope);
    if r#type == Type::Err {
        return Expr::Err;
    }

    scope.insert(name.clone(), r#type.clone());
    Expr::Decl {
        name,
        r#type,
        context: decl.text().to_owned(),
    }
}

fn parse_assig(assig: nodes::Assig, scope: &Scope) -> Expr {
    let name = assig.get_Name().to_string();
    let name_range = assig.get_Name().range();
    let value = parse_value(&assig.get_Value(), scope);
    if value == Value::Err {
        return Expr::Err;
    }
    let value_range = assig.get_Value().range();

    if let Some(var_t) = scope.get(&name).cloned() {
        let inferred = parse_type_from_value(&value, scope);
        if inferred != var_t {
            return printerr(
                &value_range,
                "wrong assignment type",
                format!("expected '{var_t}', found '{inferred}'"),
                scope,
            )
            .expr_err();
        }
    } else {
        return printerr(&name_range, "assignment to inexistent variable", "not declared", scope).expr_err();
    }

    Expr::Assig {
        name,
        value,
        context: assig.text().to_owned(),
    }
}

fn parse_list_remove_assign(listrem: nodes::ListRemAssig, scope: &Scope) -> Expr {
    let name = listrem.get_Name().to_string();
    let name_range = listrem.get_Name().range();
    let value = parse_value(&listrem.get_Value(), scope);
    if value == Value::Err {
        return Expr::Err;
    }
    let value_range = listrem.get_Value().range();
    let var_t = if let Some(t) = scope.get(&name) {
        t
    } else {
        return printerr(&name_range, "variable exists but is not a list", "not a list", scope).expr_err();
    };

    if let Type::List(lt) = var_t {
        let vt = parse_type_from_value(&value, scope);
        if **lt != vt {
            return printerr(&value_range, "wrong type", &format!("expected '{lt}', found '{vt}'"), scope).expr_err();
        }
    } else {
        return printerr(&name_range, "removing from inexistent list", "not declared", scope).expr_err();
    }

    Expr::Assig {
        name: listrem.get_Name().to_string(),
        value: Value::Op {
            op: Op::ListRemoveAll(Box::new((name, value))),
            range: value_range,
        },
        context: listrem.text().into(),
    }
}

fn parse_assig_op(name_n: nodes::Name, value_n: nodes::Value, operation: Op, context: String, scope: &Scope) -> Expr {
    let name = name_n.to_string();
    let name_range = name_n.range();
    let value = parse_value(&value_n, scope);
    if value == Value::Err {
        return Expr::Err;
    }
    let value_range = value_n.range();

    if let Some(var_t) = scope.get(&name).cloned() {
        let inferred = parse_type_from_value(&value, scope);
        if inferred != var_t {
            return printerr(
                &value_range,
                "wrong assignment type",
                format!("expected '{var_t}', found '{inferred}'"),
                scope,
            )
            .expr_err();
        }
    } else {
        return printerr(&name_range, "assignment to inexistent variable", "not declared", scope).expr_err();
    }

    let value = Value::Op {
        op: operation.set_value(
            Value::Var {
                name: name.clone(),
                range: value_range.clone(),
            },
            value,
        ),
        range: value_range,
    };

    Expr::Assig { name, value, context }
}

fn parse_fn(f: nodes::Fn, scope: &mut Scope) -> Expr {
    let name = f.get_Name().to_string();
    let r#type = if let Some(ty) = f.get_Type() {
        parse_type(&ty, scope)
    } else {
        Type::Void
    };

    let args = if let Some(args) = f.get_ArgsDef() {
        args.list_Name()
            .zip(args.list_Type())
            .map(|(name, ty)| (name.to_string(), parse_type(&ty, scope)))
            .collect()
    } else {
        Vec::new()
    };

    if scope.insert_fn(name.clone(), r#type.clone(), &args) {
        return printerr(
            &f.get_Name().range(),
            "function with the same name is defined",
            "already exists, try changing the name",
            scope,
        )
        .expr_err();
    }
    let mut fn_scope = scope.clone_into_new_scope(args.clone());
    let exprs = f.list_Expr().map(|expr| parse_expr(expr, &mut fn_scope)).collect();

    Expr::Fn {
        name,
        r#type,
        args,
        exprs,
        context: f.text().into(),
    }
}

fn parse_call(c: nodes::Call, scope: &Scope) -> Expr {
    let name = c.get_Name().to_string();
    let func = if let Some(func) = scope.get_fn(&name) {
        func
    } else {
        return printerr(&c.get_Name().range(), "call to non declared function", "does not exist", scope).expr_err();
    };

    let mut err = false;
    let mut i = 0;
    let args = c
        .list_Value()
        .map(|val| {
            let value = parse_value(&val, scope);
            let vt = parse_type_from_value(&value, scope);
            let at = &func.args[i];
            i += 1;

            if vt != *at {
                err = true;
                return printerr(&val.range(), "wrong argument type", format!("expected '{at}', found '{vt}'"), scope)
                    .value_err();
            }

            value
        })
        .collect();

    if err {
        return Expr::Err;
    }

    Expr::Call { name, args }
}

fn parse_value(value: &impl ToValueEnum, scope: &Scope) -> Value {
    match value.to_value_enum() {
        nodes::ValueChildren::Int(i) => Value::Int(i.text().parse().unwrap()),
        nodes::ValueChildren::Num(n) => Value::Num(n.text().parse().unwrap()),
        nodes::ValueChildren::Bool(b) => Value::Bool(Bool::Primitive(b.text() == "true")),
        nodes::ValueChildren::Char(c) => Value::Char(c.text().as_bytes()[0] as char),
        nodes::ValueChildren::Str(s) => {
            Value::Str(s.text().strip_prefix('"').unwrap().strip_suffix('"').unwrap().into())
        }
        nodes::ValueChildren::Tuple(t) => Value::Tuple(
            t.list_Value()
                .map(|val| parse_value(&val, scope))
                .collect::<Vec<Value>>(),
        ),
        nodes::ValueChildren::Struct(s) => Value::Struct(
            s.list_StructVal()
                .map(|sval| (sval.get_Name().text().to_owned(), parse_value(&sval.get_Value(), scope)))
                .collect(),
        ),
        nodes::ValueChildren::TupleAccess(ta) => Value::TupleAccess {
            name: ta.get_Name().text().into(),
            access_mode: match ta.get_TupleAccessType().to_enum() {
                nodes::TupleAccessTypeChildren::Name(n) => TupleAccessMode::Member(n.text().into()),
                nodes::TupleAccessTypeChildren::Index(i) => TupleAccessMode::Index(i.text().parse().unwrap()),
            },
            name_range: ta.get_Name().range(),
            access_range: ta.get_TupleAccessType().range(),
        },
        nodes::ValueChildren::List(l) => Value::List(
            l.list_Value()
                .map(|val| parse_value(&val, scope))
                .collect::<Vec<Value>>(),
        ),
        nodes::ValueChildren::ListAccess(la) => parse_value_list_access(la, scope),
        nodes::ValueChildren::Dict(d) => Value::Dict(
            d.list_DictPair()
                .map(|pair| (parse_value(&pair.get_first_Value(), scope), parse_value(&pair.get_second_Value(), scope)))
                .collect::<Vec<(Value, Value)>>(),
        ),
        nodes::ValueChildren::Name(n) => Value::Var {
            name: n.text().into(),
            range: n.range(),
        },

        // TODO! Complex values

        // TODO! Check if both values are of the same type
        nodes::ValueChildren::Op(op) => parse_value_op(op, scope),
        nodes::ValueChildren::Cmp(cmp) => Value::Bool(Bool::Cmp(parse_value_cmp(cmp, scope))),
        nodes::ValueChildren::Parenthesis(p) => Value::Parenthesis(Box::new(parse_value(&p.get_Value(), scope))),
        nodes::ValueChildren::And(and) => {
            let lhs = if let Some(lhs) = and.list_Lhs().next() {
                parse_value(&lhs, scope)
            } else {
                let cmp = and.list_Cmp().next().unwrap();
                let range = cmp.span().start()..cmp.span().end();
                Value::Cmp {cmp: parse_value_cmp(cmp, scope), range}
            };

            let rhs = parse_value(&and.get_Value(), scope);
            
            Value::Cmp { cmp: Cmp::And(Box::new((lhs, rhs))), range: and.span().start()..and.span().end() }
        }
        nodes::ValueChildren::Or(or) => {
            let lhs = if let Some(lhs) = or.list_Lhs().next() {
                parse_value(&lhs, scope)
            } else {
                let cmp = or.list_Cmp().next().unwrap();
                let range = cmp.span().start()..cmp.span().end();
                Value::Cmp {cmp: parse_value_cmp(cmp, scope), range}
            };

            let rhs = parse_value(&or.get_Value(), scope);
            
            Value::Cmp { cmp: Cmp::Or(Box::new((lhs, rhs))), range: or.span().start()..or.span().end() }
        }
        nodes::ValueChildren::Call(c) => {
            let call = parse_call(c, scope);
            if let Expr::Call { name, args } = call {
                Value::Call { name, args }
            } else {
                Value::Err
            }
        }
        nodes::ValueChildren::ModuleAccess(m) => todo!(),
        nodes::ValueChildren::TypeConversion(t) => todo!(),
    }
}

fn parse_value_list_access(la: nodes::ListAccess, scope: &Scope) -> Value {
    let name = la.get_Name().text().into();
    let name_range = la.get_Name().range();
    let access_range = {
        let range = la.get_Value().range();
        range.start - 1..range.end + 1
    };
    let list_type = if let Some(t) = scope.get(&name).cloned() {
        t
    } else {
        return printerr(&name_range, "accessed invalid list/dictionary", "list/dictionary does not exist", scope)
            .value_err();
    };

    let (access_type, access_mode) = match list_type {
        Type::List(list) => (
            *list,
            if let nodes::ValueChildren::Int(i) = la.get_Value().to_enum() {
                if let Ok(i) = i.text().parse() {
                    ListAccessMode::List(i)
                } else {
                    return printerr(
                        &access_range,
                        "negative index",
                        "index is negative, lists can only be accessed with positive numbers",
                        scope,
                    )
                    .value_err();
                }
            } else {
                return printerr(
                    &access_range,
                    "accessing list as a dictionary",
                    format!("use the index of the element you want to access instead: {name}[0]"),
                    scope,
                )
                .value_err();
            },
        ),
        Type::Dict(dict) => {
            let value = parse_value(&la.get_Value(), scope);
            let value_type = parse_type_from_value(&value, scope);
            if dict.0 != value_type {
                return printerr(
                    &access_range,
                    "wrong access type",
                    format!("expected {} found {value_type}", dict.0),
                    scope,
                )
                .value_err();
            } else {
                (dict.1, ListAccessMode::Dict(Box::new(value)))
            }
        }
        Type::Custom(_) => unreachable!(),
        _ => {
            return printerr(
                &name_range,
                "variable exists but is not a list/dictionary",
                "not a list/dictionary",
                scope,
            )
            .value_err()
        }
    };

    Value::ListAccess {
        name,
        access_type,
        access_mode,
        name_range,
        access_range,
    }
}

fn parse_value_op(op: nodes::Op, scope: &Scope) -> Value {
    Value::Op {
        op: match op.to_enum() {
            nodes::OpChildren::Add(a) => {
                Op::Add(Box::new((parse_value(&a.get_Lhs(), scope), parse_value(&a.get_Value(), scope))))
            }
            nodes::OpChildren::Sub(s) => {
                Op::Sub(Box::new((parse_value(&s.get_Lhs(), scope), parse_value(&s.get_Value(), scope))))
            }
            nodes::OpChildren::Mul(mul) => {
                Op::Mul(Box::new((parse_value(&mul.get_Lhs(), scope), parse_value(&mul.get_Value(), scope))))
            }
            nodes::OpChildren::Div(d) => {
                Op::Div(Box::new((parse_value(&d.get_Lhs(), scope), parse_value(&d.get_Value(), scope))))
            }
            nodes::OpChildren::Mod(m) => {
                Op::Mod(Box::new((parse_value(&m.get_Lhs(), scope), parse_value(&m.get_Value(), scope))))
            }
            nodes::OpChildren::Pow(p) => {
                Op::Pow(Box::new((parse_value(&p.get_Lhs(), scope), parse_value(&p.get_Value(), scope))))
            }
        },
        range: op.span().start()..op.span().end(),
    }
}

fn parse_value_cmp(cmp: nodes::Cmp, scope: &Scope) -> Cmp {
    let can_cmp = |lhs, rhs| -> bool {
        let lty = parse_type_from_value(lhs, scope);
        let rty = parse_type_from_value(rhs, scope);

        if lty == rty {
            match lty {
                Type::Tuple(_) | Type::Struct(_) => printerr(
                    &cmp.range(),
                    "comparing tuple/struct",
                    "tuples/structs cannot be compared, create a function if you need it",
                    scope,
                )
                ._false(),
                Type::Void => printerr(
                    &cmp.range(),
                    "trying to compare void expressions",
                    "functions return void, which cannot be compared",
                    scope,
                )
                ._false(),
                Type::Custom(_) => unreachable!(),
                Type::Err => false,
                _ => true,
            }
        } else {
            printerr(
                &cmp.range(),
                "comparing values of different types",
                "only comparisons of the same type are allowed",
                scope,
            )
            ._false()
        }
    };
    // TODO! Errors for all conditions
    match cmp.to_enum() {
        nodes::CmpChildren::Less(l) => {
            let lhs = parse_value(&l.get_Lhs(), scope);
            let rhs = parse_value(&l.get_Value(), scope);
            if can_cmp(&lhs, &rhs) {
                Cmp::Less(Box::new((lhs, rhs)))
            } else {
                Cmp::Err
            }
        }
        nodes::CmpChildren::Great(g) => {
            let lhs = parse_value(&g.get_Lhs(), scope);
            let rhs = parse_value(&g.get_Value(), scope);
            if can_cmp(&lhs, &rhs) {
                Cmp::Greater(Box::new((lhs, rhs)))
            } else {
                Cmp::Err
            }
        }
        nodes::CmpChildren::LessEq(le) => {
            let lhs = parse_value(&le.get_Lhs(), scope);
            let rhs = parse_value(&le.get_Value(), scope);
            if can_cmp(&lhs, &rhs) {
                Cmp::LessEq(Box::new((lhs, rhs)))
            } else {
                Cmp::Err
            }
        }
        nodes::CmpChildren::GreatEq(ge) => {
            let lhs = parse_value(&ge.get_Lhs(), scope);
            let rhs = parse_value(&ge.get_Value(), scope);
            if can_cmp(&lhs, &rhs) {
                Cmp::GreatEq(Box::new((lhs, rhs)))
            } else {
                Cmp::Err
            }
        }
        nodes::CmpChildren::Equal(eq) => {
            let lhs = parse_value(&eq.get_Lhs(), scope);
            let rhs = parse_value(&eq.get_Value(), scope);
            if can_cmp(&lhs, &rhs) {
                Cmp::Equal(Box::new((lhs, rhs)))
            } else {
                Cmp::Err
            }
        }
        nodes::CmpChildren::NotEq(neq) => {
            let lhs = parse_value(&neq.get_Lhs(), scope);
            let rhs = parse_value(&neq.get_Value(), scope);
            if can_cmp(&lhs, &rhs) {
                Cmp::NotEq(Box::new((lhs, rhs)))
            } else {
                Cmp::Err
            }
        }
        nodes::CmpChildren::Not(n) => {
            let lhs = parse_value(&n.get_Value(), scope);
            let lhs_t = parse_type_from_value(&lhs, scope);
            if lhs_t == Type::Bool {
                Cmp::Not(Box::new(lhs))
            } else if lhs_t != Type::Err {
                printerr(&(n.span().start()..n.span().end()), "wrong negation type", format!("type '{lhs_t}' can't be negated"), scope).cmp_err()
            } else {
                Cmp::Err
            }
        },
        /*
        nodes::CmpChildren::Name(name) => {
            let var = name.to_string();
            if let Some(var_t) = scope.get(&var) {
                if *var_t == Type::Bool {
                    Cmp::Name(var)
                } else {
                    printerr(&name.range(), "wrong comparison type", format!("expected 'bool', found {var_t})"), scope).cmp_err()
                }
            } else {
                printerr(&name.range(), "comparison with inexistent variable", "not declared", scope).cmp_err()
            }
        }
        nodes::CmpChildren::Bool(b) => Cmp::Bool(b.text() == "true"),
        */
    }
}

fn parse_type(ty: &nodes::Type, scope: &Scope) -> Type {
    match ty.to_enum() {
        nodes::TypeChildren::TInt(_) => Type::Int,
        nodes::TypeChildren::TNum(_) => Type::Num,
        nodes::TypeChildren::TBool(_) => Type::Bool,
        nodes::TypeChildren::TChar(_) => Type::Char,
        nodes::TypeChildren::TStr(_) => Type::Str,
        nodes::TypeChildren::TTuple(t) => {
            Type::Tuple(t.list_Type().map(|t| parse_type(&t, scope)).collect::<Vec<Type>>())
        }
        nodes::TypeChildren::TStruct(s) => Type::Struct(
            s.list_StructMem()
                .map(|mem| (mem.get_Name().text().into(), parse_type(&mem.get_Type(), scope)))
                .collect(),
        ),
        nodes::TypeChildren::TList(l) => Type::List(Box::new(parse_type(&l.get_Type(), scope))),
        nodes::TypeChildren::TDict(d) => {
            Type::Dict(Box::new((parse_type(&d.get_first_Type(), scope), parse_type(&d.get_second_Type(), scope))))
        }
        // On typedef add type to variables, then check its value and return it
        nodes::TypeChildren::TCustom(c) => {
            if let Some(r#type) = scope.get(c.text()) {
                r#type.clone()
            } else {
                printerr(&(c.span().start()..c.span().end()), "not declared type", "does not exist", scope).type_err()
            }
        }
        nodes::TypeChildren::TVoid(_) => unreachable!(),
    }
}

fn parse_type_from_value(value: &Value, scope: &Scope) -> Type {
    match value {
        Value::Int(_) => Type::Int,
        Value::Num(_) => Type::Num,
        Value::Bool(_) => Type::Bool,
        Value::Cmp { .. } => Type::Bool,
        Value::Char(_) => Type::Char,
        Value::Str(_) => Type::Str,
        Value::Tuple(t) => Type::Tuple(
            t.iter()
                .map(|val| parse_type_from_value(val, scope))
                .collect::<Vec<Type>>(),
        ),
        Value::Struct(members) => Type::Struct(
            members
                .iter()
                .map(|mem| (mem.0.clone(), parse_type_from_value(&mem.1, scope)))
                .collect(),
        ),
        Value::List(l) => Type::List(Box::new(parse_type_from_value(&l[0], scope))),
        Value::Dict(d) => {
            Type::Dict(Box::new((parse_type_from_value(&d[0].0, scope), parse_type_from_value(&d[0].1, scope))))
        }
        Value::Var { name, range } => {
            if let Some(var_t) = scope.get(name).cloned() {
                var_t
            } else {
                printerr(range, &format!("variable '{name}' does not exist"), "not declared", scope).type_err()
            }
        }
        Value::TupleAccess {
            name,
            access_mode: access_type,
            name_range,
            access_range,
        } => {
            let tuple_t = if let Some(ty) = scope.get(name) {
                ty.clone()
            } else {
                return printerr(name_range, "accessed invalid tuple/struct", "struct does not exist", scope)
                    .type_err();
            };

            match access_type {
                TupleAccessMode::Member(member) => {
                    let mut struct_t = if let Type::Struct(t) = tuple_t {
                        t
                    } else {
                        return printerr(
                            access_range,
                            "accessed tuple by member name",
                            format!("use index instead: {name}.0"),
                            scope,
                        )
                        .type_err();
                    };

                    struct_t.sort_unstable();
                    if let Ok(ty) = struct_t.binary_search_by_key(&member, |(a, _)| a) {
                        struct_t[ty].1.clone()
                    } else {
                        printerr(
                            access_range,
                            format!("member '{name}.{member}' does not exist"),
                            "not declared",
                            scope,
                        );
                        Type::Err
                    }
                }
                TupleAccessMode::Index(index) => {
                    let tuple_t = if let Type::Tuple(t) = tuple_t {
                        t
                    } else {
                        return printerr(
                            access_range,
                            "accessed struct by index",
                            format!("use member name instead: {name}.member"),
                            scope,
                        )
                        .type_err();
                    };

                    if let Some(ty) = tuple_t.get(*index) {
                        ty.clone()
                    } else {
                        let n_elems = tuple_t.len();
                        let n_elems = if n_elems > 1 {
                            format!("{} elements", n_elems)
                        } else {
                            "1 element".into()
                        };

                        printerr(
                            &(name_range.start..access_range.end),
                            "index out of bounds",
                            format!("tuple has {n_elems}, trying to access element number {}", index + 1),
                            scope,
                        )
                        .type_err()
                    }
                }
            }
        }
        Value::ListAccess { access_type, .. } => access_type.clone(),
        Value::Op { op, range } => match op {
            Op::Add(v) | Op::Sub(v) | Op::Mul(v) | Op::Div(v) | Op::Mod(v) | Op::Pow(v) => {
                parse_type_from_value(&v.0, scope)
            }
            Op::ListRemoveAll(lra) => printerr(
                range,
                "operation not permitted",
                format!("only use as expression, '{} --= ...'", lra.0),
                scope,
            )
            .type_err(),
        },
        Value::Parenthesis(p) => parse_type_from_value(p, scope),
        // TODO! Complex values
        Value::Call { name, args } => scope.get_fn_type(name),
        Value::RetExpr(_) => todo!(),
        Value::Err => Type::Err,
    }
}

#[derive(Debug)]
pub struct ParseErr;
impl ParseErr {
    pub fn type_err(self) -> Type {
        Type::Err
    }

    pub fn value_err(self) -> Value {
        Value::Err
    }

    pub fn expr_err(self) -> Expr {
        Expr::Err
    }
    pub fn cmp_err(self) -> Cmp {
        Cmp::Err
    }

    pub fn _false(self) -> bool {
        false
    }

    pub fn _true(self) -> bool {
        true
    }
}

fn printerr(range: &std::ops::Range<usize>, header: impl AsRef<str>, text: impl AsRef<str>, scope: &Scope) -> ParseErr {
    let mut list = AnnotationList::new(scope.file_path().to_string_lossy(), scope.file_as_str());
    list.error(range.start..range.end, header.as_ref(), text.as_ref())
        .unwrap();
    list.show_stderr(&Stylesheet::colored()).unwrap();
    println!();
    ParseErr
}

// NODE HELPER TRAITS
pub trait GetRange {
    fn range(&self) -> std::ops::Range<usize>;
}

impl GetRange for crate::parser::nodes::Value<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

impl GetRange for nodes::Type<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

impl GetRange for nodes::Name<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

impl GetRange for nodes::TupleAccess<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

impl GetRange for nodes::TupleAccessType<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start() - 1..self.span().end()
    }
}

impl GetRange for nodes::Cmp<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

pub trait ToValueEnum {
    fn to_value_enum(&self) -> nodes::ValueChildren<'_>;
}

impl ToValueEnum for nodes::Value<'_> {
    fn to_value_enum(&self) -> nodes::ValueChildren<'_> {
        self.to_enum()
    }
}

impl ToValueEnum for nodes::Lhs<'_> {
    fn to_value_enum(&self) -> nodes::ValueChildren<'_> {
        use nodes::LhsChildren;
        use nodes::ValueChildren;

        match self.to_enum() {
            LhsChildren::Name(n) => ValueChildren::Name(n),
            LhsChildren::Char(c) => ValueChildren::Char(c),
            LhsChildren::Bool(b) => ValueChildren::Bool(b),
            LhsChildren::TupleAccess(ta) => ValueChildren::TupleAccess(ta),
            LhsChildren::Str(s) => ValueChildren::Str(s),
            LhsChildren::Int(i) => ValueChildren::Int(i),
            LhsChildren::Call(c) => ValueChildren::Call(c),
            LhsChildren::Tuple(t) => ValueChildren::Tuple(t),
            LhsChildren::Num(n) => ValueChildren::Num(n),
            LhsChildren::Struct(s) => ValueChildren::Struct(s),
            LhsChildren::Dict(d) => ValueChildren::Dict(d),
            LhsChildren::List(l) => ValueChildren::List(l),
            LhsChildren::ListAccess(la) => ValueChildren::ListAccess(la),
            LhsChildren::Parenthesis(p) => ValueChildren::Parenthesis(p),
            LhsChildren::TypeConversion(tc) => todo!(),
            LhsChildren::ModuleAccess(ma) => todo!(),
        }
    }
}

pub trait NameNodeUtils {
    fn to_string(&self) -> String;
}

impl NameNodeUtils for nodes::Name<'_> {
    fn to_string(&self) -> String {
        self.text().to_owned()
    }
}

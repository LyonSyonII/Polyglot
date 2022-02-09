use clap::Parser as P;
use indexmap::IndexMap;
use pest::Parser as Pest;
use pest::iterators::{ Pairs, Pair };
use serde::{Deserialize, Serialize, Serializer};
use parking_lot::Mutex;

mod tree;
use tree::*;

// TODO! Check all values on list and dictionary too see if all have the same type

#[derive(clap::Parser)]
#[clap(version, about)]
struct Cli {
    file: std::path::PathBuf
}

#[derive(pest_derive::Parser, pest_typed_tree::TypedTree, Serialize, Deserialize, Debug)]
#[grammar = "grammar.pest"]
struct Parser;

lazy_static::lazy_static! {
    static ref CTX: Mutex<String> = Mutex::new(String::new());
}

fn clone_ctx() -> String {
    CTX.lock().clone()
}

fn main() {
    let cli = Cli::parse();
    let file = std::fs::read_to_string(&cli.file).unwrap();
    
    let root = nodes::Main::new(Parser::parse(Rule::Main, &file).unwrap().next().unwrap());
    
    let mut main = Main::new(Vec::new());
    let mut global = Scope::new(vec![]);
    
    for expr in root.list_Expr() {
        *CTX.lock() = expr.text().into();
        main.0.push(parse_expr(expr, &mut global));
    }
    
    let buffer = serde_yaml::to_string(&main).unwrap();
    println!("{buffer}");

    //let buffer = serde_json::to_string_pretty(&main).unwrap();
    //println!("{buffer}");
}

fn parse_expr(expr: nodes::Expr, scope: &mut Scope) -> Expr {
    match expr.to_enum() {
        nodes::ExprChildren::Assig(_) => todo!(),
        nodes::ExprChildren::Decl(_) => todo!(),
        nodes::ExprChildren::Typedef(_) => todo!(),
        nodes::ExprChildren::Init(init) => Expr::Init(parse_init(init, scope)),
    }
}

fn parse_init(init: nodes::Init, scope: &mut Scope) -> Init {
    let name = init.get_Name().text().to_owned();
    let value = parse_value(init.get_Value(), scope);
    let ctx = clone_ctx();
    
    
    let ty = match init.get_Type() {
        // Check if variable is of the same type as assignment
        Some(t) => {
            let t = parse_type(t);
            let vt = parse_type_from_value(&value, scope);
            if t != vt {
                eprintln!("ERROR: Assigned value of type {vt:?} to variable of type {t:?}.\nContext: {}", ctx);
                std::process::exit(1)
            }
            t
        },
        // No explicit type annotation, inferring type from value
        None => parse_type_from_value(&value, scope)
    };

    scope.insert(name.clone(), ty.clone());
    Init::new(name, ty, value, ctx)
}

fn parse_value(value: nodes::Value, scope: &mut Scope) -> Value {
    match value.to_enum() {
        nodes::ValueChildren::Int(i) => Value::Int(i.text().parse().unwrap()),
        nodes::ValueChildren::Num(n) => Value::Num(n.text().parse().unwrap()),
        nodes::ValueChildren::Bool(b) => {
            match b.text() {
                "true" => Value::Bool(Bool::Primitive(true)),
                "false" => Value::Bool(Bool::Primitive(false)),
                _ => Value::Bool(Bool::Cmp(parse_cmp(b.list_Cmp().next().unwrap()))),
            }
        },
        nodes::ValueChildren::Char(c) => Value::Char(c.text().as_bytes()[0] as char),
        nodes::ValueChildren::Str(s) => Value::Str(s.text().strip_prefix('"').unwrap().strip_suffix('"').unwrap().into()),
        nodes::ValueChildren::Tuple(t) => Value::Tuple(
            Tuple::new(
                t.list_Value()
                .map(|val| parse_value(val, scope))
                .collect::<Vec<Value>>()
            )
        ),
        nodes::ValueChildren::Struct(s) => Value::Struct(
            Struct::new( {
                let mut map = IndexMap::new();
                for sval in s.list_StructVal() {
                    map.insert(sval.get_Name().text().to_owned(), parse_value(sval.get_Value(), scope));
                }
                map
            }
        )),
        nodes::ValueChildren::TupleAccess(ta) => Value::TupleAccess(
            TupleAccess::new(
                ta.get_Name().text().into(), 
                    match ta.get_TupleAccessType().to_enum() {
                        nodes::TupleAccessTypeChildren::Name(n) => TupleAccessType::Member(n.text().into()),
                        nodes::TupleAccessTypeChildren::Index(i) => TupleAccessType::Index(i.text().parse().unwrap()),
                    }
            )
        ),
        nodes::ValueChildren::List(l) => Value::List(l.list_Value().map(|val| parse_value(val, scope)).collect::<Vec<Value>>()),
        
        nodes::ValueChildren::Dict(d) => {
            Value::Dict(Box::new(d.list_DictPair().map(|pair| (parse_value(pair.get_first_Value(), scope), parse_value(pair.get_second_Value(), scope))).collect::<Vec<(Value, Value)>>()))
        }
        nodes::ValueChildren::Name(n) => Value::Var(n.text().into()),
        
        // TODO! Complex values
        nodes::ValueChildren::Op(_) => todo!(),
        nodes::ValueChildren::RetExpr(_) => todo!(),
        nodes::ValueChildren::Call(_) => todo!(),
    }
}

fn parse_type(ty: nodes::Type) -> Type {
    match ty.to_enum() {
        nodes::TypeChildren::TInt(_) => Type::Int,
        nodes::TypeChildren::TNum(_) => Type::Num,
        nodes::TypeChildren::TBool(_) => Type::Bool,
        nodes::TypeChildren::TChar(_) => Type::Char,
        nodes::TypeChildren::TStr(_) => Type::Str,
        nodes::TypeChildren::TTuple(t) => Type::Tuple(
            TTuple::new(
                t.list_Type()
                .map(parse_type)
                .collect::<Vec<Type>>()
            )
        ),
        nodes::TypeChildren::TStruct(s) => Type::Struct(
            TStruct::new({
                let mut map = IndexMap::new();
                for mem in s.list_StructMem() {
                    map.insert(mem.get_Name().text().into(), parse_type(mem.get_Type()));
                }
                map
            })
        ),
        nodes::TypeChildren::TList(l) => Type::List(Box::new(parse_type(l.get_Type()))),
        
        nodes::TypeChildren::TDict(d) => {
            Type::Dict(Box::new((parse_type(d.get_first_Type()), parse_type(d.get_second_Type()))))
        },

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
        Value::Tuple(t) => {
            Type::Tuple(
                TTuple::new(
                t.values
                    .iter()
                    .map(|val| parse_type_from_value(val, scope))
                    .collect::<Vec<Type>>()
                )
            )
        },
        Value::Struct(s) => {
            Type::Struct(
                TStruct::new({
                    let mut map = IndexMap::new();
                    for mem in &s.values {
                        map.insert(mem.0.clone(), parse_type_from_value(mem.1, scope));
                    }
                    map
                })
            )
        },
        Value::List(l) => Type::List(Box::new(parse_type_from_value(&l[0], scope))),
        Value::Dict(d) => Type::Dict(Box::new((parse_type_from_value(&d[0].0, scope), parse_type_from_value(&d[0].1, scope)))),
        
        // TODO! More complex typing, based on already declared variables. I need to have a dictionary with all the variables/structs and their types
        Value::Var(var) => {
            if let Some(var_t) = scope.get(var).cloned() {
                var_t
            } else {
                eprintln!("ERROR: Variable {var} does not exist.\nContext: {}", CTX.lock());
                std::process::exit(1)
            }
        },
        Value::TupleAccess(ta) => {
            if let Some(tuple_type) = scope.get(&ta.name) {
                match &ta.access {
                    TupleAccessType::Member(m) => {
                        match tuple_type {
                            Type::Tuple(_) => {
                                eprintln!("ERROR: Tuples have no members. Use their index instead.\nContext: {}", CTX.lock());
                                std::process::exit(1)
                            },
                            Type::Struct(s) => {
                                if let Some(ty) = s.types.get(m) {
                                    ty.clone()
                                } else {
                                    eprintln!("ERROR: {}.{} does not exist.\nContext: {}", ta.name, m, CTX.lock());
                                    std::process::exit(1)
                                }
                            },
                            _ => unreachable!()
                        }
                    },
                    TupleAccessType::Index(i) => {
                        match tuple_type {
                            Type::Tuple(t) => {
                                if let Some(ty) = t.types.get(*i) {
                                    ty.clone()
                                } else {
                                    eprintln!("ERROR: Trying to access index {i} of Tuple {}, which only has {} elements.\nContext: {}", ta.name, t.types.len(), CTX.lock());
                                    std::process::exit(1)
                                }
                            },
                            Type::Struct(_) => {
                                eprintln!("ERROR: Trying to access Struct {} with index. Use the name of the members instead.\nContext: {}", ta.name, CTX.lock());
                                std::process::exit(1)
                            },
                            _ => unreachable!()
                        }
                    },
                }
            } else {
                eprintln!("ERROR: Tuple/Struct {} does not exist.\nContext: {}", ta.name, CTX.lock());
                std::process::exit(1)
            }
        },
        Value::Op(_) => todo!(),
        Value::Call(_) => todo!(),
        Value::RetExpr(_) => todo!(),
    }
}

fn parse_cmp(cmp: nodes::Cmp) -> Cmp {
    todo!()
}
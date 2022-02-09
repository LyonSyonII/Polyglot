use clap::Parser as P;
use pest::Parser as Pest;
use pest::iterators::{ Pairs, Pair };
use serde::{Deserialize, Serialize, Serializer};
mod tree;
use tree::*;

#[derive(clap::Parser)]
#[clap(version, about)]
struct Cli {
    file: std::path::PathBuf
}

#[derive(pest_derive::Parser, pest_typed_tree::TypedTree, Serialize, Deserialize, Debug)]
#[grammar = "grammar.pest"]
struct Parser;

fn main() {
    let cli = Cli::parse();
    let file = std::fs::read_to_string(&cli.file).unwrap();
    
    let root = nodes::Main::new(Parser::parse(Rule::Main, &file).unwrap().next().unwrap());
    
    let mut main = Main::new(Vec::new());
    for expr in root.list_Expr() {
        let expr = parse_expr(expr);
        main.0.push(expr);
    }
    
    let buffer = serde_yaml::to_string(&main).unwrap();
    println!("{buffer}");
}

fn parse_expr(expr: nodes::Expr) -> tree::Expr {
    match expr.to_enum() {
        nodes::ExprChildren::Assig(_) => todo!(),
        nodes::ExprChildren::Decl(_) => todo!(),
        nodes::ExprChildren::Typedef(_) => todo!(),
        nodes::ExprChildren::Init(init) => tree::Expr::Init(parse_init(init)),
    }
}

fn parse_init(init: nodes::Init) -> tree::Init {
    let name = init.get_Name().text().into();
    let value = parse_value(init.get_Value());
    
    let ty = match init.get_Type() {
        Some(t) => parse_type(t),
        None => parse_type_from_value(&value)
    };

    tree::Init::new(name, ty, value)
}

fn parse_value(value: nodes::Value) -> tree::Value {
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
        nodes::ValueChildren::Str(s) => Value::Str(s.text().into()),
        nodes::ValueChildren::Tuple(t) => Value::Tuple(
            Tuple::new(
                t.list_Value()
                .map(|val| parse_value(val))
                .collect::<Vec<Value>>()
            )
        ),
        nodes::ValueChildren::Struct(s) => Value::Struct(
            Struct::new(
                s.list_StructVal()
                    .map(|sval| (
                            sval.get_Name().text().into(), 
                            parse_value(sval.get_Value())
                        ))
                    .collect::<Vec<(String, Value)>>()
            )
        ),
        nodes::ValueChildren::TupleAccess(ta) => Value::TupleAccess(
            TupleAccess::new(
                ta.get_Name().text().into(), 
                    match ta.get_TupleAccessType().to_enum() {
                        nodes::TupleAccessTypeChildren::Name(n) => TupleAccessType::Member(n.text().into()),
                        nodes::TupleAccessTypeChildren::Index(i) => TupleAccessType::Index(i.text().parse().unwrap()),
                    }
            )
        ),
        nodes::ValueChildren::Name(n) => Value::Var(n.text().into()),

        // TODO! Complex values
        nodes::ValueChildren::RetExpr(_) => todo!(),
        nodes::ValueChildren::Call(_) => todo!(),
        nodes::ValueChildren::Op(_) => todo!(),
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
            TStruct::new(
                s.list_StructMem()
                    .map(|sm| parse_type(sm.get_Type()))
                    .collect::<Vec<Type>>()
            )
        ),
        nodes::TypeChildren::TCustom(c) => Type::Custom(c.text().into()),
    }
}

fn parse_type_from_value(value: &tree::Value) -> tree::Type {
    match value {
        tree::Value::Int(_) => tree::Type::Int,
        tree::Value::Num(_) => tree::Type::Num,
        tree::Value::Bool(_) => tree::Type::Bool,
        tree::Value::Char(_) => tree::Type::Char,
        tree::Value::Str(_) => tree::Type::Str,
        tree::Value::Tuple(t) => {
            tree::Type::Tuple(
                tree::TTuple::new(
                t.values
                    .iter()
                    .map(parse_type_from_value)
                    .collect::<Vec<tree::Type>>()
                )
            )
        },
        tree::Value::Struct(s) => {
            tree::Type::Struct(
                tree::TStruct::new(
                    s.values
                    .iter()
                        .map(|mem| parse_type_from_value(&mem.1))
                        .collect::<Vec<tree::Type>>()
                )
            )
        },

        // TODO! More complex typing, based on already declared variables. I need to have a dictionary with all the variables/structs and their types
        tree::Value::Var(var) => todo!(),
        tree::Value::TupleAccess(ta) => {
            todo!()
        },
        tree::Value::Op(_) => todo!(),
        tree::Value::Call(_) => todo!(),
        tree::Value::RetExpr(_) => todo!(),
    }
}

fn parse_cmp(cmp: nodes::Cmp) -> tree::Cmp {
    todo!()
}
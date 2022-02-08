use clap::Parser as P;
use pest::Parser as Pest;
use pest::iterators::{ Pairs, Pair };
use serde::{Deserialize, Serialize};
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
    println!("{root:#?}");
    
    let mut main = tree::Main(Vec::new());
    for expr in root.list_Expr() {
        main.0.push(parse_expr(expr));
    }
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
        Some(t) => match t.to_enum() {
            nodes::TypeChildren::TBool(_) => todo!(),
            nodes::TypeChildren::TCustom(_) => todo!(),
            nodes::TypeChildren::TNum(_) => todo!(),
            nodes::TypeChildren::TStruct(_) => todo!(),
            nodes::TypeChildren::TTuple(_) => todo!(),
            nodes::TypeChildren::TInt(_) => todo!(),
            nodes::TypeChildren::TStr(_) => todo!(),
            nodes::TypeChildren::TChar(_) => todo!(),
        },
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
        nodes::ValueChildren::Struct(s) => ,
        nodes::ValueChildren::RetExpr(_) => todo!(),

        nodes::ValueChildren::Name(n) => Value::Var(n.text().into()),
        
        nodes::ValueChildren::Call(_) => todo!(),
        nodes::ValueChildren::TupleAccess(_) => todo!(),
    
        
        
        
        nodes::ValueChildren::Op(_) => todo!(),
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
                    .map(|val| parse_type_from_value(&val))
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
        tree::Value::TupleAccess(ta) => {
            todo!()
        },
        tree::Value::Op(_) => todo!(),
        tree::Value::Call(_) => todo!(),
        tree::Value::RetExpr(_) => todo!(),
    }
}

fn parse_cmp(cmp: nodes::Cmp) -> tree::Cmp {

}
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{ops::{Add, Range}, path::PathBuf};
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, new)]
pub struct Main(pub Vec<Expr>);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Expr {
    Init {
        name: String,
        r#type: Type,
        value: Value,
        context: String
    },
    Decl {
        name: String,
        r#type: Type,
        context: String,
    },
    Assig {
        name: String,
        value: Value,
        context: String
    },
    Typedef,
    Err,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RetExpr {}

#[derive(Debug, Serialize, Deserialize, new)]
pub struct Init {
    pub name: String,
    pub r#type: Type,
    pub value: Value,
    pub context: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Num(f64),
    Bool(Bool),
    Char(char),
    Str(String),
    Tuple(Vec<Value>),
    Struct(Vec<(String, Value)>),
    TupleAccess {
        name: String,
        access_mode: TupleAccessMode,
        name_range: Range<usize>,
        access_range: Range<usize>,
    },
    List(Vec<Value>),
    ListAccess {
        name: String,
        access_mode: ListAccessMode,
        access_type: Type,
        name_range: Range<usize>,
        access_range: Range<usize>
    },
    Dict(Vec<(Value, Value)>),
    Var {
        name: String,
        range: Range<usize>,
    },
    Op {
        op: Op,
        range: Range<usize>,
    },
    Parenthesis(Box<Value>),
    Call {
        name: String,
        args: Vec<String>,
    },
    RetExpr(RetExpr),
    Err
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Bool {
    Primitive(bool),
    Cmp(Cmp),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, new)]
pub struct Call {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TupleAccessMode {
    Member(String),
    Index(usize),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ListAccessMode {
    List(usize),
    Dict(Box<Value>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Op {
    Add(Box<(Value, Value)>),
    Sub(Box<(Value, Value)>),
    ListRemoveAll(Box<(String, Value)>),
    Mul(Box<(Value, Value)>),
    Div(Box<(Value, Value)>),
    Mod(Box<(Value, Value)>),
    Pow(Box<(Value, Value)>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Cmp {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, Eq, Ord)]
pub enum Type {
    Int,
    Num,
    Bool,
    Char,
    Str,
    Tuple(Vec<Type>),
    Struct(Vec<(String, Type)>),
    List(Box<Type>),
    Dict(Box<(Type, Type)>),
    Custom(String),
    Err,
}

// CUSTOM DEFINED
pub struct Scope {
    vars: std::collections::HashMap<String, Type>,
    file: (String, PathBuf),
}

// IMPLS
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tuple(l0), Self::Tuple(r0)) => l0 == r0,
            (Self::Struct(l0), Self::Struct(r0)) => l0 == r0,
            (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
            (Self::Struct(l0), Self::Tuple(r0)) => {
                for (l0, r0) in l0.iter().zip(r0) {
                    if l0.1 != *r0 {
                        return false;
                    }
                }
                true
            }
            (Type::Int, Type::Num) | (Type::Num, Type::Int) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Num => write!(f, "num"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::Str => write!(f, "str"),
            Type::Tuple(t) => {
                let mut t = t.iter();
                write!(f, "({}", t.next().unwrap()).unwrap();
                
                for ty in t {
                    write!(f, ", {}", ty).unwrap()
                }
                write!(f, ")")
            },
            Type::Struct(s) => {
                let mut s = s.iter();
                let next = s.next().unwrap();
                write!(f, "({}: {}", next.0, next.1).unwrap();
                for (name, ty) in s {
                    write!(f, ", {name}: {ty}").unwrap()
                }
                write!(f, ")")
            },
            Type::List(l) => write!(f, "[{l}]"),
            Type::Dict(d) => write!(f, "[{} -> {}]", d.0, d.1),
            Type::Custom(c) => write!(f, "{c}"),
            Type::Err => unreachable!()
        }
    }
}

impl Scope {
    pub fn new(current_scope: Vec<(String, Type)>) -> Scope {
        let mut map = std::collections::HashMap::with_capacity(current_scope.len());
        for var in current_scope.into_iter() {
            map.insert(var.0, var.1);
        }
        
        Scope {
            vars: map,
            file: (String::new(), PathBuf::new()),
        }
    }
    
    pub fn set_file(&mut self, name: PathBuf, contents: String) {
        self.file = (contents, name);
    }
    
    pub fn file_as_str(&self) -> &str {
        self.file.0.as_str()
    }

    pub fn file_path(&self) -> &std::path::Path {
        self.file.1.as_path()
    }
}

impl std::ops::Deref for Scope {
    type Target = std::collections::HashMap<String, Type>;
    
    fn deref(&self) -> &Self::Target {
        &self.vars
    }
}

impl std::ops::DerefMut for Scope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vars
    }
}

pub trait OpUtils {
    fn set_value(self, lhs: Value, rhs: Value) -> Op;
}

impl OpUtils for Op {
    fn set_value(self, lhs: Value, rhs: Value) -> Op {
        match self {
            Op::Add(a) => Op::Add(Box::new((lhs, rhs))),
            Op::Sub(a) => Op::Sub(Box::new((lhs, rhs))),
            Op::Mul(a) => Op::Mul(Box::new((lhs, rhs))),
            Op::Div(a) => Op::Div(Box::new((lhs, rhs))),
            Op::Mod(a) => Op::Mod(Box::new((lhs, rhs))),
            Op::Pow(a) => Op::Pow(Box::new((lhs, rhs))),
            _ => unreachable!()
        }
    }
}











use std::{ops::{Add, Range}, path::PathBuf};


use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, new)]
pub struct Main(pub Vec<Expr>);

#[derive(Debug, Serialize, Deserialize)]
pub enum Expr {
    Init(Box<Init>),
    Decl,
    Assig,
    Typedef,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RetExpr {}

#[derive(Debug, Serialize, Deserialize, new)]
pub struct Init {
    pub name: String,
    pub r#type: Type,
    pub value: Value,
    //pub line_num: usize,
    pub context: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, new)]
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
    Op(Op),
    Call {
        name: String,
        args: Vec<String>,
    },
    RetExpr(RetExpr),
    Err
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Bool {
    Primitive(bool),
    Cmp(Cmp),
}

#[derive(Debug, Serialize, Deserialize, new)]
pub struct Call {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TupleAccessMode {
    Member(String),
    Index(usize),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ListAccessMode {
    List(usize),
    Dict(Box<Value>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Op {}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    ctx: String,
    file: (String, PathBuf),
    err: bool
}

// IMPLS
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tuple(l0), Self::Tuple(r0)) => l0 == r0,
            (Self::Struct(l0), Self::Struct(r0)) => l0 == r0,
            (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
            (Self::Struct(l0), Self::Tuple(r0)) => {
                for (l0, r0) in l0.into_iter().zip(r0) {
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
            Type::Tuple(t) => todo!(),
            Type::Struct(_) => todo!(),
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
            ctx: String::new(),
            file: (String::new(), PathBuf::new()),
            err: false,
        }
    }
    
    pub fn change_ctx(&mut self, new: String) {
        self.ctx = new;
    }

    pub fn ctx(&self) -> &str {
        self.ctx.as_str()
    }
    
    pub fn set_file(&mut self, name: PathBuf, contents: String) {
        self.file = (contents, name);
    }

    pub fn file(&self) -> &str {
        self.file.0.as_str()
    }

    pub fn file_name(&self) -> &std::path::Path {
        self.file.1.as_path()
    }

    pub fn err(&mut self) {
        self.err = true;
    }
    
    pub fn err_found(&self) -> bool {
        self.err
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

pub trait GetRange {
    fn range(&self) -> std::ops::Range<usize>;
}

impl GetRange for crate::nodes::Value<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

impl GetRange for crate::nodes::Type<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

impl GetRange for crate::nodes::Name<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

impl GetRange for crate::nodes::TupleAccess<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()..self.span().end()
    }
}

impl GetRange for crate::nodes::TupleAccessType<'_> {
    fn range(&self) -> std::ops::Range<usize> {
        self.span().start()-1..self.span().end()
    }
}













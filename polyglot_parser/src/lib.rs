pub mod parser;
pub mod tree;

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use crate::tree::*;
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;

    mod initialization {
        use super::*;
        use assert_fs::fixture::FileWriteStr;

        #[test]
        fn int() -> Result<(), Box<dyn std::error::Error>> {
            let file = assert_fs::NamedTempFile::new("test.txt")?;
            let code = "
            var a = 1
            var b = -1
            var c = -555
            var d = 9999999
            ";

            file.write_str(code)?;
            let out = parse(file.path(), false).unwrap();
            let correct = create_main(vec![
                create_init("a", Type::Int, Value::Int(1), "var a = 1"),
                create_init("b", Type::Int, Value::Int(-1), "var b = -1"),
                create_init("c", Type::Int, Value::Int(-555), "var c = -555"),
                create_init("d", Type::Int, Value::Int(9999999), "var d = 9999999"),
            ]);

            assert_eq!(out, correct);
            Ok(())
        }
    }

    fn create_main(vec: Vec<Expr>) -> Main {
        Main(vec)
    }

    fn create_init(name: &str, r#type: Type, value: Value, context: &str) -> Expr {
        Expr::Init {
            name: name.into(),
            r#type,
            value,
            context: context.into(),
        }
    }

    trait MainUtils {}

    impl MainUtils for Main {}
}

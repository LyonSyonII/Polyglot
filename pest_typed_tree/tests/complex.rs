use pest::Parser;
use pest_derive::Parser;
use pest_typed_tree::TypedTree;

#[derive(Parser, TypedTree)]
#[grammar = "../tests/complex.pest"]
struct P;

impl<'i> nodes::Num<'i> {
    fn value(&self) -> u64 {
        self.text().parse().unwrap()
    }
}

#[test]
fn test() {
    let data = "
CONST foo 7
SUM foofoo 3 2
CONST bar 7
SUM foobar 34 45
CONST_OPT quux 57

";
    let root_pair = P::parse(Rule::program, data)
        .map_err(|e| println!("{}", e.to_string()))
        .expect("parse failed")
        .next()
        .unwrap();
    let root_node = nodes::Program::new(root_pair);
    assert_eq!(root_node.text(), data);
    let mut sum = 0;
    for def in root_node.list_definition() {
        match def.to_enum() {
            nodes::DefinitionChildren::DefConst(def_const) => {
                sum += def_const.get_num().value();
            }
            nodes::DefinitionChildren::DefSum(def_sum) => {
                sum += def_sum.get_first_num().value();
                sum += def_sum.get_second_num().value() * 2;
            }
            nodes::DefinitionChildren::DefOpt(def_opt) => {
                sum += def_opt.get_num().map(|num| num.value()).unwrap_or(0);
            }
        }
    }
    assert_eq!(sum, 7 + 7 + 7 + 124 + 57);
}

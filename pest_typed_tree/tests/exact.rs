use pest::Parser;
use pest_derive::Parser;
use pest_typed_tree::TypedTree;

#[derive(Parser, TypedTree)]
#[grammar = "../tests/exact.pest"]
struct P;

#[test]
fn test() {
    let data = "hello John";
    let root_pair = P::parse(Rule::greeting, data)
        .expect("parse failed")
        .next()
        .unwrap();
    let root_node = nodes::Greeting::new(root_pair);
    assert_eq!(root_node.text(), data);
    assert_eq!(root_node.get_greeting_kind().text(), "hello");
    assert_eq!(root_node.get_name_reference().text(), "John");
}

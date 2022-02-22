use pest::Parser;
use pest_derive::Parser;
use pest_typed_tree::TypedTree;

#[derive(Parser, TypedTree)]
#[grammar = "../tests/simple.pest"]
struct P;

#[test]
fn test() {
    let data = "hello   world";
    let root_pair = P::parse(Rule::hello_world, data)
        .expect("parse failed")
        .next()
        .unwrap();
    let root_node = nodes::HelloWorld::new(root_pair);
    assert_eq!(root_node.text(), data);
    let world_node = root_node.get_world();
    assert_eq!(world_node.text(), "world");
}

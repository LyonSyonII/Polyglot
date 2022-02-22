extern crate proc_macro;

mod generate;

/// Generates helper structs for type-safe AST handling.
/// # Structs
/// For each rule `pest-typed-tree` will generate struct.
/// E.g. `foo_rule` -> `FooRule`, and so on.
/// This struct is wrapper for pest [`Pair`](pest::iterators::Pair) for which
/// as_rule() == foo_rule holds.
/// You can create this struct using new() constructor, and get underlying 
/// pair or text.
/// # Typed getters
/// `pest-typed-tree` will generate getters that return child nodes in AST.
/// Consider following pest grammar:
/// ```pest
/// foo = ...
/// bar = ...
/// baz = {foo ~ bar?}
/// ```
/// Following APIs will be generated:
/// ```rust,ignore
/// impl Baz {
///     fn get_foo(&self) -> Foo;
///     fn get_bar(&self) -> Option<Bar>;
/// }
/// ```
/// # Converting to enum
/// If rule is just choice of several unique rules, `pest-typed-tree` will 
/// generate `to_enum` function that returns enum with actual AST child.
/// For following grammar:
/// ```pest
/// foo = ...
/// bar = ...
/// baz = ...
/// quux = {foo | bar | baz}
/// ```
/// Following API will be available
/// ```rust,ignore
/// enum QuuxChildren {
///     Foo(Foo),
///     Bar(Bar),
///     Baz(Baz),
/// }
/// impl Quux {
///     fn to_enum(&self) -> QuuxChildren;
/// }
/// ```
/// See test `complex.rs` for typed getters and enum conversion usage examples.

#[proc_macro_derive(TypedTree, attributes(grammar))]
pub fn derive_typed_tree(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(stream as syn::DeriveInput);
    if input.attrs.len() != 1 {
        panic!(r#"derive input must contain exactly once #[grammar = "path/to/pest/grammar""#)
    }
    let attr = input.attrs.into_iter().next().unwrap();
    let mut gram_path = attr.tokens.to_string();
    // FIXME: dirty hack
    gram_path = gram_path.trim_start_matches('=').trim().replace('"', "");
    // following is copy-pasted from pest_generator
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let gram_path = std::path::Path::new(&root).join("src/").join(&gram_path);

    let grammar = std::fs::read_to_string(&gram_path).unwrap_or_else(|err| {
        panic!(
            "failed to read parser grammar file {}: {}",
            gram_path.display(),
            err
        )
    });
    generate::generate(grammar)
}

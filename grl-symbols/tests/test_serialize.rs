#![allow(unused)]
mod common;
use common::*;
use grl_symbols::prelude::ScopeSyntax;

#[test]
fn test() {
    let mut tree = Tree::new();

    let root = tree.get_root_scope_id();

    let sym = tree
        .create_symbol_in_scope(root, "gaz")
        .expect("Can't create lable");
    tree.set_value_for_id(sym, ValueType::Num(100))
        .expect("Can't set symbol value");

    let x = tree.to_json().unwrap();

    println!("{}", x);

    let y = Tree::from_json(x.as_str()).unwrap();

    let restored = y
        .get_symbol_info_from_name("::gaz")
        .expect("symbol should survive round trip");
    assert!(matches!(restored.value, Some(ValueType::Num(100))));
}

#[test]
fn custom_scope_syntax_survives_round_trip() {
    let mut tree = Tree::with_syntax(ScopeSyntax::new("."));
    let root = tree.get_root_scope_id();
    let module = tree.create_or_get_scope_for_parent("module", root).unwrap();
    let symbol = tree.create_symbol_in_scope(module, "label").unwrap();
    tree.set_value_for_id(symbol, ValueType::Num(7)).unwrap();

    let restored = Tree::from_json(&tree.to_json().unwrap()).unwrap();
    let info = restored.get_symbol_info_from_name(".module.label").unwrap();
    assert_eq!(info.scoped_name(), ".module.label");
    assert!(matches!(info.value, Some(ValueType::Num(7))));
}

#[test]
fn tree_implements_generic_serde_round_trip() {
    let mut tree = Tree::new();
    let root = tree.get_root_scope_id();
    let symbol = tree.create_symbol_in_scope(root, "value").unwrap();
    tree.set_value_for_id(symbol, ValueType::Num(42)).unwrap();

    let encoded = serde_json::to_string(&tree).unwrap();
    let restored: Tree = serde_json::from_str(&encoded).unwrap();
    let info = restored.get_symbol_info_from_name("::value").unwrap();
    assert!(matches!(info.value, Some(ValueType::Num(42))));
}

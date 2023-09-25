#![allow(unused)]
mod common;
use common::*;


#[cfg(test)]
fn test() {
    let mut tree = Tree::new();

    let root = tree.get_root_scope_id();

    let sym = tree.create_symbol_in_scope(root, "gaz").expect("Can't create lable");
    tree.set_value_for_id(sym, ValueType::Num(100)).expect("Can't set symbol value");

    let x = tree.to_json();

    println!("{}",x);

    let y = Tree::from_json(x.as_str());

    assert!(false);
}

use grl_symbols::prelude::*;
use std::hint::black_box;
use std::time::Instant;

fn build_tree() -> SymbolTree<u64, u64, u64> {
    let mut tree = SymbolTree::new();
    let root = tree.get_root_scope_id();

    for scope_index in 0..256 {
        let scope = tree
            .create_or_get_scope_for_parent(&format!("scope_{scope_index}"), root)
            .unwrap();
        for symbol_index in 0..32 {
            let symbol = tree
                .create_symbol_in_scope(scope, &format!("symbol_{symbol_index}"))
                .unwrap();
            tree.set_value_for_id(symbol, symbol_index).unwrap();
        }
    }

    tree
}

fn main() {
    let tree = build_tree();
    let iterations = 1_000_000u64;
    let start = Instant::now();

    for _ in 0..iterations {
        let result = tree.get_symbol_info_from_name("::scope_255::symbol_31");
        black_box(result).unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "qualified lookup: {iterations} queries in {elapsed:?} ({:.1} M queries/s)",
        iterations as f64 / elapsed.as_secs_f64() / 1_000_000.0
    );
}

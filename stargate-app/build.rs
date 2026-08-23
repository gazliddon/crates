use std::path::PathBuf;

fn main() {
    let parser_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../gazm/gazm-plugin/treesitter-gazm/src");
    println!(
        "cargo:rerun-if-changed={}",
        parser_dir.join("parser.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        parser_dir.join("tree_sitter/parser.h").display()
    );

    cc::Build::new()
        .file(parser_dir.join("parser.c"))
        .include(parser_dir)
        .warnings(false)
        .compile("tree_sitter_gazm");
}

use crux_core::typegen::TypeGen;
use shared::address::{AddressSuggestion, AddressSuggestionsResult};
use shared::events::{FieldIdent, FormViewModel};
use shared::App;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=../shared");

    let mut gen = TypeGen::new();

    gen.register_app::<App>()?;
    gen.register_type::<FieldIdent>()?;
    gen.register_type::<AddressSuggestion>()?;
    gen.register_type::<AddressSuggestionsResult>()?;
    gen.register_type::<FormViewModel>()?;

    let output_root = PathBuf::from("./generated");

    gen.swift("SharedTypes", output_root.join("swift"))?;

    gen.java("com.crux.example.counter", output_root.join("java"))?;

    gen.typescript("shared_types", output_root.join("typescript"))?;

    Ok(())
}

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
mod attrs;
mod config;

#[proc_macro_derive(Config, attributes(config))]
pub fn config(input: TokenStream) -> TokenStream {
  match config::config(parse_macro_input!(input as DeriveInput)) {
    Ok(out) => out.into(),
    Err(e) => e.to_compile_error().into(),
  }
}

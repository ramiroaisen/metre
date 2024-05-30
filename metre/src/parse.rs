//! Utility functions to use with `#[config(parse_env)]` attribute

use std::str::FromStr;

/// Utility function to use with `#[config(parse_env)]` attribute
///
/// the function will return a [`Vec<T>`] from a comma separated env string
///
/// the type `T` must implement [`FromStr`]
///
/// currently Rust is not smart enough to infer the type `T` from the context, so you have to specify it explicitly
///
/// usage:
///
/// ```text
/// #[config(merge = metre::parse::comma_separated::<T>)]
/// my_field: Vec<T>
/// ```
pub fn comma_separated<T: FromStr>(value: &str) -> Result<Option<Vec<T>>, T::Err> {
  let mut target = vec![];
  if !value.is_empty() {
    for item in value.split(',') {
      let parsed = item.parse::<T>()?;
      target.push(parsed);
    }
  }

  Ok(Some(target))
}

//! Utility functions to use with `#[config(merge)]` attribute

use std::convert::Infallible;

/// Utility function to use with `#[config(merge)]` attribute
///
/// this function will append a vector to the previos one instead of replacing it
///
/// usage:
///
/// ```text
/// #[config(merge = metre::merge::append_vec)]
/// my_field: Vec<T>
/// ```
pub fn append_vec<T>(left: &mut Option<Vec<T>>, right: Option<Vec<T>>) -> Result<(), Infallible> {
  if let Some(left_vec) = left {
    if let Some(mut right_vec) = right {
      left_vec.append(&mut right_vec);
    }
  } else if let Some(right_vec) = right {
    *left = Some(right_vec);
  };

  Ok(())
}

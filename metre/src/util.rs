#![doc(hidden)]

use crate::error::MergeError;
use crate::PartialConfig;
use std::convert::Infallible;

pub trait UnOption {
  type T;
}

impl<T> UnOption for Option<T> {
  type T = T;
}

#[inline(always)]
pub fn merge_flat<T>(left: &mut Option<T>, right: Option<T>) -> Result<(), Infallible> {
  if let Some(right) = right {
    *left = Some(right)
  }
  Ok(())
}

#[inline(always)]
pub fn merge_nested<T: PartialConfig>(left: &mut T, right: T) -> Result<(), MergeError> {
  left.merge(right)
}

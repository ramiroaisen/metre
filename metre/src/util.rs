#![doc(hidden)]

use std::convert::Infallible;
use crate::PartialConfig;
use crate::error::MergeError;

#[doc(hidden)]
pub trait UnOption {
  type T;
}

#[doc(hidden)]
impl<T> UnOption for Option<T> {
  type T = T;
}

#[doc(hidden)]
#[inline(always)]
pub fn merge_flat<T>(left: &mut Option<T>, right: Option<T>) -> Result<(), Infallible> {
  if let Some(right) = right {
    *left = Some(right) 
  }
  Ok(())
}

#[doc(hidden)]
#[inline(always)]
pub fn merge_nested<T: PartialConfig>(left: &mut T, right: T) -> Result<(), MergeError> {
  left.merge(right)
}      
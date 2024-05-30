extern crate creusot_contracts;
use creusot_contracts::*;

#[ensures((^x) == 0i32)]
pub fn set_zero(x: &mut i32) {
  *x = 0;
}
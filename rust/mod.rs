#![allow(internal_features)]
#![feature(ptr_metadata)]
#![feature(stmt_expr_attributes)]
#![feature(new_range_api)]
#![feature(unsize)]
#![feature(core_intrinsics)]
#![feature(coerce_unsized)]
#![no_std]
#![no_implicit_prelude]

pub mod prelude;
mod real_main;
pub mod std;
mod sys;
pub mod util;

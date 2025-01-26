#![no_std]
#![feature(new_range_api)]
#![feature(unsize)]
#![feature(stmt_expr_attributes)]
#![feature(coerce_unsized)]
#![allow(internal_features)]
#![feature(core_intrinsics)]
#![no_implicit_prelude]

#[macro_use]
pub mod std;

mod crypto;
pub mod disk;
pub mod net;
pub mod prelude;
mod real_main;
mod sys;
pub mod util;

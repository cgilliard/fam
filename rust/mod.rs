#![no_std]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![allow(internal_features)]
#![feature(core_intrinsics)]
#![no_implicit_prelude]

#[macro_use]
pub mod std;

pub mod disk;
pub mod net;
pub mod prelude;
mod real_main;
mod sys;
pub mod util;

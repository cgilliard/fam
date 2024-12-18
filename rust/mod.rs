#![allow(internal_features)]
#![feature(ptr_metadata)]
#![feature(unsize)]
#![feature(core_intrinsics)]
#![feature(coerce_unsized)]
#![no_std]
#![no_implicit_prelude]

use crate::std::boxed::Box;
use crate::std::clone::Clone;
use crate::std::error::Error;
use crate::std::error::ErrorKind::*;
use crate::std::option::{Option, Option::None, Option::Some};
use crate::std::result::{Result, Result::Err, Result::Ok};

mod real_main;
pub mod std;
pub mod sys;

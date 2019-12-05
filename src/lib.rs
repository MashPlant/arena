#![doc(include = "../readme.md")]

#![feature(ptr_internals)]
#![feature(dropck_eyepatch)]
#![feature(vec_into_raw_parts)]
#![feature(external_doc)]
#![deny(missing_docs)]
#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

/// Providing struct `SimpleArena`.
pub mod simple;
/// Providing struct `Arena`.
pub mod arena;

pub use crate::{simple::SimpleArena, arena::Arena};
//! Utilities for writing middleware
//!
#![doc = include_str!("../docs/middleware.md")]

mod from_fn;

pub use self::from_fn::{from_fn, FromFn, FromFnLayer, Next};

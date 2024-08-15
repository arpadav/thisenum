#![doc = include_str!("../README.md")]
// --------------------------------------------------
// external
// --------------------------------------------------
use thiserror::Error;
pub use thisenum_impl::*;

#[derive(Error, Debug)]
/// All errors that can occur while using [`TryFrom`]
/// implementation for [`Const`]
pub enum Error {
    #[error("Unable to convert `{0}` to `{1}`")]
    InvalidValue(String, String),
    #[error("Multiple associated enum arms defined with value `{0}`")]
    UnreachableValue(String),
    #[error("Unable to return variant `{0}` from constant, since the variant has nested arguments")]
    UnableToReturnVariant(String),
}
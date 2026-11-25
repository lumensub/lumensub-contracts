//! Types shared by the LumenSub contracts and the off-chain services that read them.
//!
//! Everything in this crate is part of the public interface. Error codes and enum
//! discriminants are stable: new values are only ever appended.
#![no_std]

mod error;

pub use error::Error;

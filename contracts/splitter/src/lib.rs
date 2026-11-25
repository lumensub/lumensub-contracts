//! Revenue splits for LumenSub merchants.
//!
//! A split is a list of recipients and their shares in basis points, adding up to
//! 10,000. The split's owner approves this contract to spend a token, and `distribute`
//! pulls an amount from the owner and pays each recipient their share. A merchant who
//! wants revenue shared automatically sets the owner wallet as their LumenSub payout
//! and has a keeper call `distribute` after each charge.
#![no_std]

mod storage;
mod types;

pub use types::{Split, SplitError};

use soroban_sdk::contract;

#[contract]
pub struct SplitterContract;

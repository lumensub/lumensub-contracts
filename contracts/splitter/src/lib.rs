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
mod validation;

#[cfg(test)]
mod test;

pub use types::{Split, SplitError};

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec};

/// Most recipients a single split can have.
pub const MAX_RECIPIENTS: u32 = 10;
/// Shares must add up to exactly this many basis points.
pub const TOTAL_BPS: u32 = 10_000;

#[contract]
pub struct SplitterContract;

#[contractimpl]
impl SplitterContract {
    /// Creates a split owned by `owner` and returns its id.
    pub fn create_split(
        env: Env,
        owner: Address,
        recipients: Vec<(Address, u32)>,
    ) -> Result<u64, SplitError> {
        owner.require_auth();
        validation::check_recipients(&recipients)?;
        let id = storage::next_id(&env);
        let split = Split {
            id,
            owner,
            recipients,
        };
        storage::write_split(&env, &split);
        let topics = (symbol_short!("split_new"), id);
        env.events().publish(topics, split.owner);
        Ok(id)
    }

    /// Returns a split.
    pub fn get_split(env: Env, split_id: u64) -> Result<Split, SplitError> {
        storage::read_split(&env, split_id)
    }
}

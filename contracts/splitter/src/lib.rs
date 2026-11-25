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

use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, Vec};

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

    /// Replaces the recipient list. Owner only. The new list is checked the same way as
    /// on creation.
    pub fn update_split(
        env: Env,
        owner: Address,
        split_id: u64,
        recipients: Vec<(Address, u32)>,
    ) -> Result<(), SplitError> {
        owner.require_auth();
        let mut split = storage::read_split(&env, split_id)?;
        if split.owner != owner {
            return Err(SplitError::Unauthorized);
        }
        validation::check_recipients(&recipients)?;
        split.recipients = recipients;
        storage::write_split(&env, &split);
        let topics = (symbol_short!("split_upd"), split_id);
        env.events().publish(topics, owner);
        Ok(())
    }

    /// Returns a split.
    pub fn get_split(env: Env, split_id: u64) -> Result<Split, SplitError> {
        storage::read_split(&env, split_id)
    }

    /// Pulls `amount` of `token` from the split owner and pays each recipient their share.
    ///
    /// Anyone may call this; the owner controls how much can move through the allowance
    /// they give this contract. Shares round down and the last recipient receives the
    /// remainder, so the full amount is always paid out.
    pub fn distribute(
        env: Env,
        split_id: u64,
        token: Address,
        amount: i128,
    ) -> Result<(), SplitError> {
        if amount <= 0 {
            return Err(SplitError::InvalidAmount);
        }
        let split = storage::read_split(&env, split_id)?;
        let client = token::Client::new(&env, &token);
        let spender = env.current_contract_address();

        let owner = &split.owner;
        let count = split.recipients.len();
        let mut paid: i128 = 0;
        for i in 0..count {
            let (recipient, bps) = split.recipients.get_unchecked(i);
            let share = if i + 1 == count {
                amount - paid
            } else {
                amount * i128::from(bps) / i128::from(TOTAL_BPS)
            };
            if share > 0 {
                client.transfer_from(&spender, owner, &recipient, &share);
            }
            paid += share;
        }

        let topics = (symbol_short!("split_out"), split_id);
        env.events().publish(topics, (token, amount));
        Ok(())
    }
}

//! The LumenSub subscription contract.
//!
//! Merchants register and publish plans. Subscribers approve the contract to spend a
//! token on their behalf and subscribe to a plan. From then on anyone can call `charge`
//! once a payment is due; the caller (a "keeper") earns a small reward for doing so.
#![no_std]

mod events;
mod governance;
mod payments;
mod pricing;
mod storage;

#[cfg(test)]
mod test;

pub use lumensub_types::*;

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct SubscriptionContract;

// Setup and administration.
#[contractimpl]
impl SubscriptionContract {
    /// Sets the admin, the treasury and the fee rates. Can only be called once.
    ///
    /// `fee_bps` is capped at 1000 (10%) and `keeper_reward_bps` at 500 (5%).
    pub fn initialize(
        env: Env,
        admin: Address,
        treasury: Address,
        fee_bps: u32,
        keeper_reward_bps: u32,
    ) -> Result<(), Error> {
        governance::initialize(&env, admin, treasury, fee_bps, keeper_reward_bps)
    }

    /// Returns the protocol settings.
    pub fn get_config(env: Env) -> Result<Config, Error> {
        storage::read_config(&env)
    }

    /// Changes the protocol fee. Admin only, capped at 1000 bps.
    pub fn set_fee(env: Env, admin: Address, fee_bps: u32) -> Result<(), Error> {
        governance::set_fee(&env, &admin, fee_bps)
    }

    /// Changes the reward paid to keepers. Admin only, capped at 500 bps.
    pub fn set_keeper_reward(env: Env, admin: Address, reward_bps: u32) -> Result<(), Error> {
        governance::set_keeper_reward(&env, &admin, reward_bps)
    }

    /// Changes where protocol fees are sent. Admin only.
    pub fn set_treasury(env: Env, admin: Address, treasury: Address) -> Result<(), Error> {
        governance::set_treasury(&env, &admin, treasury)
    }

    /// Stops new plans, new subscriptions, charges and resumes. Cancelling and
    /// pausing a subscription still work, so subscribers can always get out.
    pub fn pause_protocol(env: Env, admin: Address) -> Result<(), Error> {
        governance::set_paused(&env, &admin, true)
    }

    /// Lifts a protocol pause.
    pub fn unpause_protocol(env: Env, admin: Address) -> Result<(), Error> {
        governance::set_paused(&env, &admin, false)
    }

    /// Starts handing the admin role to `new_admin`, who must then call `accept_admin`.
    pub fn propose_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        governance::propose_admin(&env, &admin, &new_admin)
    }

    /// Completes an admin handover started with `propose_admin`.
    pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        governance::accept_admin(&env, &new_admin)
    }
}

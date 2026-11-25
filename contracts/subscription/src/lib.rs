//! The LumenSub subscription contract.
//!
//! Merchants register and publish plans. Subscribers approve the contract to spend a
//! token on their behalf and subscribe to a plan. From then on anyone can call `charge`
//! once a payment is due; the caller (a "keeper") earns a small reward for doing so.
#![no_std]
// `create_plan` takes the full set of plan terms as separate arguments so the contract
// interface stays flat and easy to call from any client.
#![allow(clippy::too_many_arguments)]

mod billing;
mod events;
mod governance;
mod merchants;
mod payments;
mod plans;
mod pricing;
mod storage;
mod subscriptions;

#[cfg(test)]
mod test;

pub use lumensub_types::*;

use plans::PlanTerms;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

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

    /// Replaces the contract code with a Wasm blob already uploaded to the network.
    /// Admin only. Storage is kept, so new code must read the existing layout.
    pub fn upgrade(env: Env, admin: Address, wasm_hash: BytesN<32>) -> Result<(), Error> {
        governance::upgrade(&env, &admin, wasm_hash)
    }
}

// Merchants.
#[contractimpl]
impl SubscriptionContract {
    /// Registers `merchant` and sets where its revenue is paid. Calling it again
    /// updates the payout address and keeps everything else.
    pub fn register_merchant(env: Env, merchant: Address, payout: Address) -> Result<(), Error> {
        merchants::register(&env, &merchant, &payout)
    }

    /// Returns a merchant's record.
    pub fn get_merchant(env: Env, merchant: Address) -> Result<Merchant, Error> {
        storage::read_merchant(&env, &merchant)
    }
}

// Plans.
#[contractimpl]
impl SubscriptionContract {
    /// Publishes a plan and returns its id. The merchant must be registered.
    ///
    /// `amount` is charged every `period_secs` (at least one hour). `trial_secs` delays
    /// the first charge. `max_cycles` limits the number of charges, with zero meaning no
    /// limit. `grace_secs` is how long a failed charge is retried before the
    /// subscription lapses.
    pub fn create_plan(
        env: Env,
        merchant: Address,
        token: Address,
        amount: i128,
        period_secs: u64,
        trial_secs: u64,
        max_cycles: u32,
        grace_secs: u64,
    ) -> Result<u64, Error> {
        let terms = PlanTerms {
            token,
            amount,
            period_secs,
            trial_secs,
            max_cycles,
            grace_secs,
        };
        plans::create(&env, &merchant, terms)
    }

    /// Opens or closes a plan to new subscribers. Merchant only.
    pub fn set_plan_active(
        env: Env,
        merchant: Address,
        plan_id: u64,
        active: bool,
    ) -> Result<(), Error> {
        plans::set_active(&env, &merchant, plan_id, active)
    }

    /// Returns a plan.
    pub fn get_plan(env: Env, plan_id: u64) -> Result<Plan, Error> {
        storage::read_plan(&env, plan_id)
    }

    /// Number of plans created so far. Plan ids run from 1 to this number.
    pub fn plan_count(env: Env) -> u64 {
        storage::plan_count(&env)
    }
}

// Subscriptions.
#[contractimpl]
impl SubscriptionContract {
    /// Subscribes to a plan and returns the new subscription id.
    ///
    /// Before calling this, the subscriber approves this contract as a spender of the
    /// plan's token, for at least one period and ideally for many. Without a trial the
    /// first period is charged straight away.
    pub fn subscribe(env: Env, subscriber: Address, plan_id: u64) -> Result<u64, Error> {
        subscriptions::subscribe(&env, &subscriber, plan_id)
    }

    /// Returns a subscription.
    pub fn get_subscription(env: Env, sub_id: u64) -> Result<Subscription, Error> {
        storage::read_subscription(&env, sub_id)
    }

    /// Number of subscriptions created so far. Ids run from 1 to this number.
    pub fn subscription_count(env: Env) -> u64 {
        storage::subscription_count(&env)
    }

    /// Pauses billing. Subscriber only. Works even while the protocol is paused.
    pub fn pause(env: Env, subscriber: Address, sub_id: u64) -> Result<(), Error> {
        subscriptions::pause(&env, &subscriber, sub_id)
    }

    /// Resumes a paused subscription. The time spent paused is added to the next
    /// billing date. Subscriber only.
    pub fn resume(env: Env, subscriber: Address, sub_id: u64) -> Result<(), Error> {
        subscriptions::resume(&env, &subscriber, sub_id)
    }

    /// Cancels a subscription for good. Either the subscriber or the merchant may call
    /// this, and it works even while the protocol is paused.
    pub fn cancel(env: Env, caller: Address, sub_id: u64) -> Result<(), Error> {
        subscriptions::cancel(&env, &caller, sub_id)
    }

    /// Switches to another plan from the same merchant in the same token. The unused
    /// part of the current period is credited against the first charge on the new plan,
    /// which is due immediately. Subscriber only.
    pub fn change_plan(
        env: Env,
        subscriber: Address,
        sub_id: u64,
        new_plan_id: u64,
    ) -> Result<(), Error> {
        subscriptions::change_plan(&env, &subscriber, sub_id, new_plan_id)
    }
}

// Billing.
#[contractimpl]
impl SubscriptionContract {
    /// Charges a subscription that is due and returns the amount taken.
    ///
    /// Anyone can call this. The caller is paid the keeper reward out of the charge.
    /// If the subscriber's allowance or balance is too low the charge is recorded as
    /// failed, the subscription goes past due, and the call returns zero instead of
    /// failing, so the new status is saved.
    pub fn charge(env: Env, keeper: Address, sub_id: u64) -> Result<i128, Error> {
        billing::charge(&env, &keeper, sub_id)
    }

    /// True when `charge` would attempt a payment right now.
    pub fn is_due(env: Env, sub_id: u64) -> bool {
        billing::is_due(&env, sub_id)
    }
}

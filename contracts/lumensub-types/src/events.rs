//! Data payloads of the events the subscription contract publishes.
//!
//! Each event's first topic is a short symbol (for example `charged`) and the second
//! topic is the plan or subscription id. The structs below are the event data.

use soroban_sdk::{contracttype, Address, Symbol};

use crate::SubscriptionStatus;

/// Data of `("plan_new", plan_id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanCreatedEvent {
    pub merchant: Address,
    pub token: Address,
    pub amount: i128,
    pub period_secs: u64,
}

/// Data of `("sub_new", sub_id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscribedEvent {
    pub plan_id: u64,
    pub subscriber: Address,
}

/// Data of `("charged", sub_id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChargedEvent {
    /// Total taken from the subscriber.
    pub amount: i128,
    /// Part of `amount` sent to the treasury.
    pub fee: i128,
    /// Part of `amount` sent to the keeper.
    pub keeper_reward: i128,
    /// The cycle this charge paid for, starting at 1.
    pub cycle: u32,
    pub next_charge_at: u64,
}

/// Data of `("chg_fail", sub_id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChargeFailedEvent {
    /// `allowance` or `balance`.
    pub reason: Symbol,
    /// Status after the failed attempt: `PastDue` or `Lapsed`.
    pub status: SubscriptionStatus,
}

/// Data of `("cancelled", sub_id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelledEvent {
    /// The subscriber or the merchant.
    pub by: Address,
}

/// Data of `("plan_chg", sub_id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanChangedEvent {
    pub from_plan: u64,
    pub to_plan: u64,
}

/// Data of `("refunded", sub_id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundedEvent {
    pub amount: i128,
}

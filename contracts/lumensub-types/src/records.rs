use soroban_sdk::{contracttype, Address};

/// Protocol-wide settings, stored once in instance storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// Account allowed to change settings, pause the protocol and upgrade the contract.
    pub admin: Address,
    /// Receives the protocol fee from every charge.
    pub treasury: Address,
    /// Protocol fee in basis points of each charge. At most 1000 (10%).
    pub fee_bps: u32,
    /// Reward paid to the keeper that triggers a charge, in basis points. At most 500 (5%).
    pub keeper_reward_bps: u32,
    /// When true, new plans, new subscriptions and charges are rejected.
    pub paused: bool,
}

/// A business that sells plans.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merchant {
    /// Where the merchant's share of each charge is sent. Can be a wallet or a split owner.
    pub payout: Address,
    /// Number of plans this merchant has created.
    pub plan_count: u32,
    /// Ledger timestamp of the first registration.
    pub registered_at: u64,
}

/// Something a merchant sells on a schedule.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub id: u64,
    pub merchant: Address,
    /// Stellar asset contract the plan is priced in.
    pub token: Address,
    /// Price per period, in the token's smallest unit.
    pub amount: i128,
    /// Length of one billing period in seconds.
    pub period_secs: u64,
    /// Free trial before the first charge, in seconds. Zero means no trial.
    pub trial_secs: u64,
    /// Number of periods to charge before the subscription completes. Zero means no limit.
    pub max_cycles: u32,
    /// How long a failed charge can be retried before the subscription lapses, in seconds.
    pub grace_secs: u64,
    /// Inactive plans keep billing existing subscribers but accept no new ones.
    pub active: bool,
    pub created_at: u64,
    /// Subscriptions currently attached to this plan, including paused and past-due ones.
    pub subscriber_count: u32,
}

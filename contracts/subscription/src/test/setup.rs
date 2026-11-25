//! Shared fixtures: a deployed and initialized contract, a token, and the usual cast of
//! accounts.

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

use crate::{SubscriptionContract, SubscriptionContractClient};

pub const FEE_BPS: u32 = 100;
pub const KEEPER_BPS: u32 = 50;
/// One unit of a token with seven decimals.
pub const PRICE: i128 = 10_000_000;
pub const DAY: u64 = 86_400;
pub const MONTH: u64 = 30 * DAY;
/// Ledger timestamp every test starts at.
pub const START: u64 = 1_750_000_000;

pub struct Setup<'a> {
    pub env: Env,
    pub client: SubscriptionContractClient<'a>,
    pub admin: Address,
    pub treasury: Address,
    pub merchant: Address,
    pub payout: Address,
    pub subscriber: Address,
    pub keeper: Address,
    pub token: TokenClient<'a>,
    pub token_admin: StellarAssetClient<'a>,
}

impl Setup<'_> {
    /// Deploys and initializes the contract. No merchant is registered yet.
    pub fn bare() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|li| li.timestamp = START);

        let admin = Address::generate(&env);
        let treasury = Address::generate(&env);
        let merchant = Address::generate(&env);
        let payout = Address::generate(&env);
        let subscriber = Address::generate(&env);
        let keeper = Address::generate(&env);

        let asset = env.register_stellar_asset_contract_v2(admin.clone());
        let token = TokenClient::new(&env, &asset.address());
        let token_admin = StellarAssetClient::new(&env, &asset.address());
        token_admin.mint(&subscriber, &(PRICE * 100));

        let contract_id = env.register(SubscriptionContract, ());
        let client = SubscriptionContractClient::new(&env, &contract_id);
        client.initialize(&admin, &treasury, &FEE_BPS, &KEEPER_BPS);

        Setup {
            env,
            client,
            admin,
            treasury,
            merchant,
            payout,
            subscriber,
            keeper,
            token,
            token_admin,
        }
    }

    /// Deploys, initializes and registers the merchant.
    pub fn with_merchant() -> Self {
        let s = Self::bare();
        s.client.register_merchant(&s.merchant, &s.payout);
        s
    }

    /// Creates a monthly plan at `PRICE` with the given trial, cycle limit and grace.
    pub fn create_plan(&self, trial_secs: u64, max_cycles: u32, grace_secs: u64) -> u64 {
        self.client.create_plan(
            &self.merchant,
            &self.token.address,
            &PRICE,
            &MONTH,
            &trial_secs,
            &max_cycles,
            &grace_secs,
        )
    }

    /// Lets the contract pull up to `amount` from the default subscriber.
    pub fn approve(&self, amount: i128) {
        let expiration = self.env.ledger().sequence() + 10_000;
        self.token
            .approve(&self.subscriber, &self.client.address, &amount, &expiration);
    }

    /// Approves two years of charges and subscribes the default subscriber.
    pub fn subscribe(&self, plan_id: u64) -> u64 {
        self.approve(PRICE * 24);
        self.client.subscribe(&self.subscriber, &plan_id)
    }

    /// Moves the ledger clock forward.
    pub fn advance(&self, secs: u64) {
        self.env.ledger().with_mut(|li| li.timestamp += secs);
    }

    pub fn balance(&self, account: &Address) -> i128 {
        self.token.balance(account)
    }
}

/// A deployed contract that has not been initialized.
pub fn uninitialized() -> (Env, SubscriptionContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SubscriptionContract, ());
    let client = SubscriptionContractClient::new(&env, &contract_id);
    (env, client)
}

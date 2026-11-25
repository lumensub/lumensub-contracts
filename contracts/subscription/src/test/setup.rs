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
}

/// A deployed contract that has not been initialized.
pub fn uninitialized() -> (Env, SubscriptionContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SubscriptionContract, ());
    let client = SubscriptionContractClient::new(&env, &contract_id);
    (env, client)
}

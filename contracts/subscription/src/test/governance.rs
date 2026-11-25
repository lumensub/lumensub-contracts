use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{uninitialized, Setup, FEE_BPS, KEEPER_BPS};
use crate::{Config, Error};

#[test]
fn initialize_stores_the_config() {
    let s = Setup::bare();
    let config = s.client.get_config();
    assert_eq!(
        config,
        Config {
            admin: s.admin.clone(),
            treasury: s.treasury.clone(),
            fee_bps: FEE_BPS,
            keeper_reward_bps: KEEPER_BPS,
            paused: false,
        }
    );
}

#[test]
fn initialize_only_works_once() {
    let s = Setup::bare();
    let result = s
        .client
        .try_initialize(&s.admin, &s.treasury, &FEE_BPS, &KEEPER_BPS);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn initialize_rejects_a_fee_above_ten_percent() {
    let (env, client) = uninitialized();
    let admin = Address::generate(&env);
    let result = client.try_initialize(&admin, &admin, &1_001, &0);
    assert_eq!(result, Err(Ok(Error::FeeTooHigh)));
}

#[test]
fn initialize_rejects_a_keeper_reward_above_five_percent() {
    let (env, client) = uninitialized();
    let admin = Address::generate(&env);
    let result = client.try_initialize(&admin, &admin, &0, &501);
    assert_eq!(result, Err(Ok(Error::FeeTooHigh)));
}

#[test]
fn reading_config_before_initialize_fails() {
    let (_env, client) = uninitialized();
    assert_eq!(client.try_get_config(), Err(Ok(Error::NotInitialized)));
}

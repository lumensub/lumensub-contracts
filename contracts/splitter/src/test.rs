use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env, Vec};

use crate::{SplitError, SplitterContract, SplitterContractClient};

struct Setup<'a> {
    env: Env,
    client: SplitterContractClient<'a>,
    owner: Address,
    alice: Address,
    bob: Address,
}

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SplitterContract, ());
    let client = SplitterContractClient::new(&env, &contract_id);
    Setup {
        owner: Address::generate(&env),
        alice: Address::generate(&env),
        bob: Address::generate(&env),
        client,
        env,
    }
}

fn shares(s: &Setup, alice_bps: u32, bob_bps: u32) -> Vec<(Address, u32)> {
    let mut list = Vec::new(&s.env);
    list.push_back((s.alice.clone(), alice_bps));
    list.push_back((s.bob.clone(), bob_bps));
    list
}

#[test]
fn creating_a_split_stores_it() {
    let s = setup();
    let id = s.client.create_split(&s.owner, &shares(&s, 7_000, 3_000));
    assert_eq!(id, 1);
    let split = s.client.get_split(&id);
    assert_eq!(split.owner, s.owner);
    assert_eq!(split.recipients, shares(&s, 7_000, 3_000));
    assert_eq!(s.client.create_split(&s.owner, &shares(&s, 1, 9_999)), 2);
}

#[test]
fn shares_must_add_up_to_ten_thousand() {
    let s = setup();
    assert_eq!(
        s.client.try_create_split(&s.owner, &shares(&s, 5_000, 4_999)),
        Err(Ok(SplitError::InvalidShares))
    );
}

#[test]
fn zero_shares_are_rejected() {
    let s = setup();
    assert_eq!(
        s.client.try_create_split(&s.owner, &shares(&s, 10_000, 0)),
        Err(Ok(SplitError::InvalidShares))
    );
}

#[test]
fn recipients_must_be_distinct() {
    let s = setup();
    let mut twice = Vec::new(&s.env);
    twice.push_back((s.alice.clone(), 5_000));
    twice.push_back((s.alice.clone(), 5_000));
    assert_eq!(
        s.client.try_create_split(&s.owner, &twice),
        Err(Ok(SplitError::DuplicateRecipient))
    );
}

#[test]
fn at_most_ten_recipients_are_allowed() {
    let s = setup();
    let mut many = Vec::new(&s.env);
    for _ in 0..11 {
        many.push_back((Address::generate(&s.env), 1_000));
    }
    assert_eq!(
        s.client.try_create_split(&s.owner, &many),
        Err(Ok(SplitError::TooManyRecipients))
    );
}

#[test]
fn an_empty_split_is_rejected() {
    let s = setup();
    assert_eq!(
        s.client.try_create_split(&s.owner, &Vec::new(&s.env)),
        Err(Ok(SplitError::NoRecipients))
    );
}

#[test]
fn unknown_splits_are_not_found() {
    let s = setup();
    assert_eq!(
        s.client.try_get_split(&50),
        Err(Ok(SplitError::SplitNotFound))
    );
}

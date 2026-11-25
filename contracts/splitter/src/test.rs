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
        s.client
            .try_create_split(&s.owner, &shares(&s, 5_000, 4_999)),
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

/// Mints `amount` to the owner and approves the splitter to spend it.
fn fund_owner(s: &Setup, amount: i128) -> TokenClient<'static> {
    let issuer = Address::generate(&s.env);
    let asset = s.env.register_stellar_asset_contract_v2(issuer);
    let issuer_client = StellarAssetClient::new(&s.env, &asset.address());
    issuer_client.mint(&s.owner, &amount);
    let token = TokenClient::new(&s.env, &asset.address());
    let exp = s.env.ledger().sequence() + 1_000;
    token.approve(&s.owner, &s.client.address, &amount, &exp);
    token
}

#[test]
fn distributing_pays_each_recipient_their_share() {
    let s = setup();
    let id = s.client.create_split(&s.owner, &shares(&s, 7_000, 3_000));
    let token = fund_owner(&s, 1_000);

    s.client.distribute(&id, &token.address, &1_000);
    assert_eq!(token.balance(&s.alice), 700);
    assert_eq!(token.balance(&s.bob), 300);
    assert_eq!(token.balance(&s.owner), 0);
}

#[test]
fn rounding_dust_goes_to_the_last_recipient() {
    let s = setup();
    let carol = Address::generate(&s.env);
    let mut three = shares(&s, 3_333, 3_333);
    three.push_back((carol.clone(), 3_334));
    let id = s.client.create_split(&s.owner, &three);
    let token = fund_owner(&s, 100);

    s.client.distribute(&id, &token.address, &100);
    assert_eq!(token.balance(&s.alice), 33);
    assert_eq!(token.balance(&s.bob), 33);
    assert_eq!(token.balance(&carol), 34);
}

#[test]
fn distributing_nothing_is_rejected() {
    let s = setup();
    let id = s.client.create_split(&s.owner, &shares(&s, 5_000, 5_000));
    let token = fund_owner(&s, 100);
    assert_eq!(
        s.client.try_distribute(&id, &token.address, &0),
        Err(Ok(SplitError::InvalidAmount))
    );
}

#[test]
fn the_owner_can_change_the_shares() {
    let s = setup();
    let id = s.client.create_split(&s.owner, &shares(&s, 5_000, 5_000));
    s.client
        .update_split(&s.owner, &id, &shares(&s, 2_500, 7_500));
    assert_eq!(s.client.get_split(&id).recipients, shares(&s, 2_500, 7_500));
}

#[test]
fn only_the_owner_can_change_the_shares() {
    let s = setup();
    let id = s.client.create_split(&s.owner, &shares(&s, 5_000, 5_000));
    assert_eq!(
        s.client
            .try_update_split(&s.alice, &id, &shares(&s, 9_000, 1_000)),
        Err(Ok(SplitError::Unauthorized))
    );
}

#[test]
fn updates_are_validated_like_new_splits() {
    let s = setup();
    let id = s.client.create_split(&s.owner, &shares(&s, 5_000, 5_000));
    assert_eq!(
        s.client
            .try_update_split(&s.owner, &id, &shares(&s, 5_000, 6_000)),
        Err(Ok(SplitError::InvalidShares))
    );
}

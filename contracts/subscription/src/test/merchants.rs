use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{Setup, START};
use crate::Error;

#[test]
fn registering_stores_the_payout_address() {
    let s = Setup::with_merchant();
    let record = s.client.get_merchant(&s.merchant);
    assert_eq!(record.payout, s.payout);
    assert_eq!(record.plan_count, 0);
    assert_eq!(record.registered_at, START);
}

#[test]
fn registering_again_updates_the_payout() {
    let s = Setup::with_merchant();
    let new_payout = Address::generate(&s.env);
    s.client.register_merchant(&s.merchant, &new_payout);
    assert_eq!(s.client.get_merchant(&s.merchant).payout, new_payout);
}

#[test]
fn unknown_merchants_are_not_found() {
    let s = Setup::with_merchant();
    let stranger = Address::generate(&s.env);
    assert_eq!(
        s.client.try_get_merchant(&stranger),
        Err(Ok(Error::MerchantNotFound))
    );
}

#[test]
fn registration_is_blocked_while_paused() {
    let s = Setup::bare();
    s.client.pause_protocol(&s.admin);
    assert_eq!(
        s.client.try_register_merchant(&s.merchant, &s.payout),
        Err(Ok(Error::ProtocolPaused))
    );
}

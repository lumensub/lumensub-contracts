use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{Setup, DAY, PRICE};
use crate::Error;

/// Gives the merchant tokens of its own to refund from.
fn fund_merchant(s: &Setup) {
    s.token_admin.mint(&s.merchant, &(PRICE * 10));
}

#[test]
fn a_merchant_can_refund_part_of_a_payment() {
    let s = Setup::with_merchant();
    fund_merchant(&s);
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    let before = s.balance(&s.subscriber);

    s.client.refund(&s.merchant, &id, &(PRICE / 2));
    assert_eq!(s.balance(&s.subscriber) - before, PRICE / 2);
    assert_eq!(s.client.get_subscription(&id).total_refunded, PRICE / 2);
}

#[test]
fn refunds_cannot_exceed_what_was_paid() {
    let s = Setup::with_merchant();
    fund_merchant(&s);
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.refund(&s.merchant, &id, &(PRICE / 2));
    assert_eq!(
        s.client.try_refund(&s.merchant, &id, &(PRICE / 2 + 1)),
        Err(Ok(Error::RefundTooLarge))
    );
}

#[test]
fn nothing_can_be_refunded_during_a_trial() {
    let s = Setup::with_merchant();
    fund_merchant(&s);
    let id = s.subscribe(s.create_plan(7 * DAY, 0, DAY));
    assert_eq!(
        s.client.try_refund(&s.merchant, &id, &1),
        Err(Ok(Error::RefundTooLarge))
    );
}

#[test]
fn only_the_merchant_can_refund() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    let stranger = Address::generate(&s.env);
    assert_eq!(
        s.client.try_refund(&stranger, &id, &1),
        Err(Ok(Error::MerchantMismatch))
    );
}

#[test]
fn refunds_must_be_positive() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    assert_eq!(
        s.client.try_refund(&s.merchant, &id, &0),
        Err(Ok(Error::InvalidAmount))
    );
}

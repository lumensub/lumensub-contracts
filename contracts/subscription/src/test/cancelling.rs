use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{Setup, DAY, MONTH};
use crate::{Error, SubscriptionStatus};

#[test]
fn the_subscriber_can_cancel() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 0, DAY);
    let id = s.subscribe(plan);
    s.client.cancel(&s.subscriber, &id);

    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::Cancelled
    );
    assert_eq!(s.client.get_plan(&plan).subscriber_count, 0);
}

#[test]
fn the_merchant_can_cancel() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.cancel(&s.merchant, &id);
    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::Cancelled
    );
}

#[test]
fn nobody_else_can_cancel() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    let stranger = Address::generate(&s.env);
    assert_eq!(
        s.client.try_cancel(&stranger, &id),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn a_cancelled_subscription_is_never_charged_again() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.cancel(&s.subscriber, &id);
    s.advance(MONTH);
    assert!(!s.client.is_due(&id));
    assert_eq!(
        s.client.try_charge(&s.keeper, &id),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn cancelling_twice_fails() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.cancel(&s.subscriber, &id);
    assert_eq!(
        s.client.try_cancel(&s.subscriber, &id),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn cancelling_works_during_a_protocol_pause() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.pause_protocol(&s.admin);
    s.client.cancel(&s.subscriber, &id);
    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::Cancelled
    );
}

#[test]
fn a_paused_subscription_can_be_cancelled() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.pause(&s.subscriber, &id);
    s.client.cancel(&s.subscriber, &id);
    assert_eq!(s.client.get_subscription(&id).paused_at, 0);
}

use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{Setup, DAY, MONTH, PRICE, START};
use crate::{Error, SubscriptionStatus};

#[test]
fn a_missing_allowance_makes_the_subscription_past_due() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, 3 * DAY));
    s.approve(0);
    s.advance(MONTH);

    assert_eq!(s.client.charge(&s.keeper, &id), 0);
    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::PastDue);
    assert_eq!(sub.past_due_since, START + MONTH);
    assert_eq!(sub.cycles_charged, 1);
    assert_eq!(s.balance(&s.keeper), 0);
}

#[test]
fn a_missing_balance_makes_the_subscription_past_due() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, 3 * DAY));
    let elsewhere = Address::generate(&s.env);
    s.token
        .transfer(&s.subscriber, &elsewhere, &s.balance(&s.subscriber));
    s.advance(MONTH);

    assert_eq!(s.client.charge(&s.keeper, &id), 0);
    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::PastDue
    );
}

#[test]
fn paying_up_within_the_grace_period_recovers_the_subscription() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, 3 * DAY));
    s.approve(0);
    s.advance(MONTH);
    s.client.charge(&s.keeper, &id);

    s.advance(2 * DAY);
    s.approve(PRICE * 12);
    assert!(s.client.is_due(&id));
    assert_eq!(s.client.charge(&s.keeper, &id), PRICE);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::Active);
    assert_eq!(sub.past_due_since, 0);
    assert_eq!(sub.cycles_charged, 2);
    // The new period starts from the day the payment went through.
    assert_eq!(sub.next_charge_at, START + MONTH + 2 * DAY + MONTH);
}

#[test]
fn the_subscription_lapses_once_the_grace_period_runs_out() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 0, 3 * DAY);
    let id = s.subscribe(plan);
    s.approve(0);
    s.advance(MONTH);
    s.client.charge(&s.keeper, &id);

    s.advance(3 * DAY);
    assert_eq!(s.client.charge(&s.keeper, &id), 0);
    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::Lapsed
    );
    assert_eq!(s.client.get_plan(&plan).subscriber_count, 0);
    assert!(!s.client.is_due(&id));
    assert_eq!(
        s.client.try_charge(&s.keeper, &id),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn with_no_grace_period_a_failed_charge_lapses_immediately() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, 0));
    s.approve(0);
    s.advance(MONTH);
    s.client.charge(&s.keeper, &id);
    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::Lapsed
    );
}

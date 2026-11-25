use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{Setup, DAY, MONTH, PRICE, START};
use crate::{Error, SubscriptionStatus};

#[test]
fn subscribing_without_a_trial_pays_the_first_period() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 0, DAY);
    let id = s.subscribe(plan);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::Active);
    assert_eq!(sub.cycles_charged, 1);
    assert_eq!(sub.total_paid, PRICE);
    assert_eq!(sub.last_charged_at, START);
    assert_eq!(sub.next_charge_at, START + MONTH);

    // 1% protocol fee, no keeper on sign-up.
    assert_eq!(s.balance(&s.payout), 9_900_000);
    assert_eq!(s.balance(&s.treasury), 100_000);
    assert_eq!(s.balance(&s.subscriber), PRICE * 99);
}

#[test]
fn subscribing_with_a_trial_charges_nothing_yet() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(14 * DAY, 0, DAY);
    let id = s.subscribe(plan);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::Trialing);
    assert_eq!(sub.cycles_charged, 0);
    assert_eq!(sub.next_charge_at, START + 14 * DAY);
    assert_eq!(s.balance(&s.payout), 0);
}

#[test]
fn subscribing_records_both_sides() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 0, DAY);
    let id = s.subscribe(plan);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.plan_id, plan);
    assert_eq!(sub.subscriber, s.subscriber);
    assert_eq!(sub.merchant, s.merchant);
    assert_eq!(sub.token, s.token.address);
    assert_eq!(s.client.get_plan(&plan).subscriber_count, 1);
    assert_eq!(s.client.subscription_count(), 1);
}

#[test]
fn subscribing_needs_an_allowance() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 0, DAY);
    assert_eq!(
        s.client.try_subscribe(&s.subscriber, &plan),
        Err(Ok(Error::InsufficientAllowance))
    );
}

#[test]
fn subscribing_needs_a_balance() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 0, DAY);
    let broke = Address::generate(&s.env);
    let expiration = s.env.ledger().sequence() + 1_000;
    s.token
        .approve(&broke, &s.client.address, &PRICE, &expiration);
    assert_eq!(
        s.client.try_subscribe(&broke, &plan),
        Err(Ok(Error::InsufficientBalance))
    );
}

#[test]
fn closed_plans_take_no_new_subscribers() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 0, DAY);
    s.client.set_plan_active(&s.merchant, &plan, &false);
    s.approve(PRICE);
    assert_eq!(
        s.client.try_subscribe(&s.subscriber, &plan),
        Err(Ok(Error::PlanInactive))
    );
}

#[test]
fn a_single_cycle_plan_completes_on_sign_up() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 1, DAY);
    let id = s.subscribe(plan);
    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::Completed
    );
}

#[test]
fn unknown_subscriptions_are_not_found() {
    let s = Setup::with_merchant();
    assert_eq!(
        s.client.try_get_subscription(&9),
        Err(Ok(Error::SubscriptionNotFound))
    );
}

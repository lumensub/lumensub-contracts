use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{Setup, DAY, MONTH, PRICE, START};
use crate::Error;

#[test]
fn plans_get_sequential_ids() {
    let s = Setup::with_merchant();
    assert_eq!(s.create_plan(0, 0, DAY), 1);
    assert_eq!(s.create_plan(0, 0, DAY), 2);
    assert_eq!(s.client.plan_count(), 2);
    assert_eq!(s.client.get_merchant(&s.merchant).plan_count, 2);
}

#[test]
fn a_new_plan_stores_its_terms() {
    let s = Setup::with_merchant();
    let id = s.create_plan(7 * DAY, 12, 3 * DAY);
    let plan = s.client.get_plan(&id);
    assert_eq!(plan.merchant, s.merchant);
    assert_eq!(plan.token, s.token.address);
    assert_eq!(plan.amount, PRICE);
    assert_eq!(plan.period_secs, MONTH);
    assert_eq!(plan.trial_secs, 7 * DAY);
    assert_eq!(plan.max_cycles, 12);
    assert_eq!(plan.grace_secs, 3 * DAY);
    assert!(plan.active);
    assert_eq!(plan.created_at, START);
    assert_eq!(plan.subscriber_count, 0);
}

#[test]
fn unregistered_merchants_cannot_create_plans() {
    let s = Setup::bare();
    let result = s
        .client
        .try_create_plan(&s.merchant, &s.token.address, &PRICE, &MONTH, &0, &0, &0);
    assert_eq!(result, Err(Ok(Error::MerchantNotFound)));
}

#[test]
fn the_price_must_be_positive() {
    let s = Setup::with_merchant();
    let result = s
        .client
        .try_create_plan(&s.merchant, &s.token.address, &0, &MONTH, &0, &0, &0);
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn periods_shorter_than_an_hour_are_rejected() {
    let s = Setup::with_merchant();
    let result = s
        .client
        .try_create_plan(&s.merchant, &s.token.address, &PRICE, &59, &0, &0, &0);
    assert_eq!(result, Err(Ok(Error::InvalidPeriod)));
}

#[test]
fn trials_and_grace_periods_are_capped() {
    let s = Setup::with_merchant();
    let long_trial = 366 * DAY;
    let long_grace = 31 * DAY;
    let token = s.token.address.clone();
    assert_eq!(
        s.client
            .try_create_plan(&s.merchant, &token, &PRICE, &MONTH, &long_trial, &0, &0),
        Err(Ok(Error::InvalidPeriod))
    );
    assert_eq!(
        s.client
            .try_create_plan(&s.merchant, &token, &PRICE, &MONTH, &0, &0, &long_grace),
        Err(Ok(Error::InvalidPeriod))
    );
}

#[test]
fn merchants_can_close_a_plan_to_new_subscribers() {
    let s = Setup::with_merchant();
    let id = s.create_plan(0, 0, DAY);
    s.client.set_plan_active(&s.merchant, &id, &false);
    assert!(!s.client.get_plan(&id).active);
    s.client.set_plan_active(&s.merchant, &id, &true);
    assert!(s.client.get_plan(&id).active);
}

#[test]
fn only_the_owning_merchant_can_close_a_plan() {
    let s = Setup::with_merchant();
    let id = s.create_plan(0, 0, DAY);
    let other = Address::generate(&s.env);
    assert_eq!(
        s.client.try_set_plan_active(&other, &id, &false),
        Err(Ok(Error::MerchantMismatch))
    );
}

#[test]
fn unknown_plans_are_not_found() {
    let s = Setup::with_merchant();
    assert_eq!(s.client.try_get_plan(&42), Err(Ok(Error::PlanNotFound)));
}

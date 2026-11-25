use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{Setup, DAY, MONTH, START};
use crate::{Error, SubscriptionStatus};

#[test]
fn a_paused_subscription_is_not_charged() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.pause(&s.subscriber, &id);
    s.advance(MONTH);

    assert!(!s.client.is_due(&id));
    assert_eq!(
        s.client.try_charge(&s.keeper, &id),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn resuming_adds_the_paused_time_to_the_billing_date() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.advance(10 * DAY);
    s.client.pause(&s.subscriber, &id);
    s.advance(20 * DAY);
    s.client.resume(&s.subscriber, &id);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::Active);
    assert_eq!(sub.paused_at, 0);
    assert_eq!(sub.next_charge_at, START + MONTH + 20 * DAY);
}

#[test]
fn a_paused_trial_resumes_as_a_trial() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(7 * DAY, 0, DAY));
    s.client.pause(&s.subscriber, &id);
    s.advance(DAY);
    s.client.resume(&s.subscriber, &id);
    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::Trialing
    );
}

#[test]
fn only_the_subscriber_can_pause() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    let stranger = Address::generate(&s.env);
    assert_eq!(
        s.client.try_pause(&stranger, &id),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn resuming_needs_a_paused_subscription() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    assert_eq!(
        s.client.try_resume(&s.subscriber, &id),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn pausing_works_during_a_protocol_pause_but_resuming_does_not() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.pause_protocol(&s.admin);
    s.client.pause(&s.subscriber, &id);
    assert_eq!(
        s.client.try_resume(&s.subscriber, &id),
        Err(Ok(Error::ProtocolPaused))
    );
}

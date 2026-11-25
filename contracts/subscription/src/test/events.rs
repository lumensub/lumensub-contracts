//! Indexers depend on the exact topics and payloads, so they are pinned here.

use soroban_sdk::testutils::Events;
use soroban_sdk::{symbol_short, IntoVal, Symbol, TryFromVal, Val, Vec};

use super::setup::{Setup, DAY, MONTH, PRICE, START};
use crate::{
    CancelledEvent, ChargeFailedEvent, ChargedEvent, PlanCreatedEvent, SubscribedEvent,
    SubscriptionStatus,
};

/// Topics and data of the last event the subscription contract published.
fn last_event(s: &Setup) -> (Vec<Val>, Val) {
    let (contract, topics, data) = s.env.events().all().last().unwrap();
    assert_eq!(contract, s.client.address);
    (topics, data)
}

fn topics(s: &Setup, name: Symbol, id: u64) -> Vec<Val> {
    (name, id).into_val(&s.env)
}

#[test]
fn creating_a_plan_publishes_plan_new() {
    let s = Setup::with_merchant();
    let id = s.create_plan(0, 0, DAY);
    let (event_topics, data) = last_event(&s);
    assert_eq!(event_topics, topics(&s, symbol_short!("plan_new"), id));
    assert_eq!(
        PlanCreatedEvent::try_from_val(&s.env, &data).unwrap(),
        PlanCreatedEvent {
            merchant: s.merchant.clone(),
            token: s.token.address.clone(),
            amount: PRICE,
            period_secs: MONTH,
        }
    );
}

#[test]
fn subscribing_publishes_sub_new() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(7 * DAY, 0, DAY);
    let id = s.subscribe(plan);
    let (event_topics, data) = last_event(&s);
    assert_eq!(event_topics, topics(&s, symbol_short!("sub_new"), id));
    assert_eq!(
        SubscribedEvent::try_from_val(&s.env, &data).unwrap(),
        SubscribedEvent {
            plan_id: plan,
            subscriber: s.subscriber.clone(),
        }
    );
}

#[test]
fn charging_publishes_charged() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.advance(MONTH);
    s.client.charge(&s.keeper, &id);
    let (event_topics, data) = last_event(&s);
    assert_eq!(event_topics, topics(&s, symbol_short!("charged"), id));
    assert_eq!(
        ChargedEvent::try_from_val(&s.env, &data).unwrap(),
        ChargedEvent {
            amount: PRICE,
            fee: 100_000,
            keeper_reward: 50_000,
            cycle: 2,
            next_charge_at: START + 2 * MONTH,
        }
    );
}

#[test]
fn a_failed_charge_publishes_chg_fail() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.approve(0);
    s.advance(MONTH);
    s.client.charge(&s.keeper, &id);
    let (event_topics, data) = last_event(&s);
    assert_eq!(event_topics, topics(&s, symbol_short!("chg_fail"), id));
    assert_eq!(
        ChargeFailedEvent::try_from_val(&s.env, &data).unwrap(),
        ChargeFailedEvent {
            reason: symbol_short!("allowance"),
            status: SubscriptionStatus::PastDue,
        }
    );
}

#[test]
fn cancelling_publishes_who_cancelled() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.client.cancel(&s.merchant, &id);
    let (event_topics, data) = last_event(&s);
    assert_eq!(event_topics, topics(&s, symbol_short!("cancelled"), id));
    assert_eq!(
        CancelledEvent::try_from_val(&s.env, &data).unwrap(),
        CancelledEvent {
            by: s.merchant.clone(),
        }
    );
}

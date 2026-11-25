//! Event publishing. Topics are `(name, id)` and the data is one of the structs from
//! `lumensub-types`, so indexers can decode every event with the same schema.

use lumensub_types::{
    CancelledEvent, ChargeFailedEvent, ChargedEvent, Config, PlanChangedEvent, PlanCreatedEvent,
    RefundedEvent, SubscribedEvent, SubscriptionStatus,
};
use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub(crate) fn config_updated(env: &Env, config: &Config) {
    let topics = (symbol_short!("config"),);
    env.events().publish(topics, config.clone());
}

pub(crate) fn merchant_registered(env: &Env, merchant: &Address, payout: &Address) {
    let topics = (symbol_short!("merchant"), merchant.clone());
    env.events().publish(topics, payout.clone());
}

pub(crate) fn plan_created(env: &Env, plan_id: u64, data: PlanCreatedEvent) {
    let topics = (symbol_short!("plan_new"), plan_id);
    env.events().publish(topics, data);
}

pub(crate) fn plan_status(env: &Env, plan_id: u64, active: bool) {
    let topics = (symbol_short!("plan_act"), plan_id);
    env.events().publish(topics, active);
}

pub(crate) fn subscribed(env: &Env, sub_id: u64, plan_id: u64, subscriber: &Address) {
    let topics = (symbol_short!("sub_new"), sub_id);
    let data = SubscribedEvent {
        plan_id,
        subscriber: subscriber.clone(),
    };
    env.events().publish(topics, data);
}

pub(crate) fn charged(env: &Env, sub_id: u64, data: ChargedEvent) {
    let topics = (symbol_short!("charged"), sub_id);
    env.events().publish(topics, data);
}

pub(crate) fn charge_failed(env: &Env, sub_id: u64, reason: Symbol, status: SubscriptionStatus) {
    let topics = (symbol_short!("chg_fail"), sub_id);
    env.events()
        .publish(topics, ChargeFailedEvent { reason, status });
}

pub(crate) fn paused(env: &Env, sub_id: u64) {
    let topics = (symbol_short!("paused"), sub_id);
    env.events().publish(topics, ());
}

pub(crate) fn resumed(env: &Env, sub_id: u64) {
    let topics = (symbol_short!("resumed"), sub_id);
    env.events().publish(topics, ());
}

pub(crate) fn cancelled(env: &Env, sub_id: u64, by: &Address) {
    let topics = (symbol_short!("cancelled"), sub_id);
    env.events()
        .publish(topics, CancelledEvent { by: by.clone() });
}

pub(crate) fn plan_changed(env: &Env, sub_id: u64, from_plan: u64, to_plan: u64) {
    let topics = (symbol_short!("plan_chg"), sub_id);
    env.events()
        .publish(topics, PlanChangedEvent { from_plan, to_plan });
}

pub(crate) fn refunded(env: &Env, sub_id: u64, amount: i128) {
    let topics = (symbol_short!("refunded"), sub_id);
    env.events().publish(topics, RefundedEvent { amount });
}

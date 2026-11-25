use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::setup::{Setup, DAY, MONTH, PRICE, START};
use crate::{Error, SubscriptionStatus};

fn plan_priced_at(s: &Setup, amount: i128) -> u64 {
    s.client
        .create_plan(&s.merchant, &s.token.address, &amount, &MONTH, &0, &0, &DAY)
}

#[test]
fn upgrading_credits_the_unused_days() {
    let s = Setup::with_merchant();
    let basic = s.create_plan(0, 0, DAY);
    let pro = plan_priced_at(&s, PRICE * 2);
    let id = s.subscribe(basic);

    s.advance(10 * DAY);
    s.client.change_plan(&s.subscriber, &id, &pro);

    // Twenty of thirty days were unused.
    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.plan_id, pro);
    assert_eq!(sub.credit, 6_666_666);
    assert_eq!(sub.next_charge_at, START + 10 * DAY);
    assert!(s.client.is_due(&id));

    assert_eq!(s.client.charge(&s.keeper, &id), PRICE * 2 - 6_666_666);
    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.credit, 0);
    assert_eq!(sub.next_charge_at, START + 10 * DAY + MONTH);
}

#[test]
fn downgrading_carries_leftover_credit_forward() {
    let s = Setup::with_merchant();
    let basic = s.create_plan(0, 0, DAY);
    let lite = plan_priced_at(&s, PRICE / 4);
    let id = s.subscribe(basic);

    s.advance(10 * DAY);
    s.client.change_plan(&s.subscriber, &id, &lite);
    assert_eq!(s.client.charge(&s.keeper, &id), 0);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::Active);
    assert_eq!(sub.cycles_charged, 2);
    assert_eq!(sub.credit, 6_666_666 - PRICE / 4);
}

#[test]
fn changing_plan_during_a_trial_keeps_the_trial() {
    let s = Setup::with_merchant();
    let trial = s.create_plan(7 * DAY, 0, DAY);
    let other = plan_priced_at(&s, PRICE * 3);
    let id = s.subscribe(trial);
    s.client.change_plan(&s.subscriber, &id, &other);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::Trialing);
    assert_eq!(sub.credit, 0);
    assert_eq!(sub.next_charge_at, START + 7 * DAY);
}

#[test]
fn plan_changes_move_the_seat() {
    let s = Setup::with_merchant();
    let basic = s.create_plan(0, 0, DAY);
    let pro = plan_priced_at(&s, PRICE * 2);
    let id = s.subscribe(basic);
    s.client.change_plan(&s.subscriber, &id, &pro);
    assert_eq!(s.client.get_plan(&basic).subscriber_count, 0);
    assert_eq!(s.client.get_plan(&pro).subscriber_count, 1);
}

#[test]
fn switching_to_the_same_plan_fails() {
    let s = Setup::with_merchant();
    let basic = s.create_plan(0, 0, DAY);
    let id = s.subscribe(basic);
    assert_eq!(
        s.client.try_change_plan(&s.subscriber, &id, &basic),
        Err(Ok(Error::SamePlan))
    );
}

#[test]
fn switching_to_another_merchants_plan_fails() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    let rival = Address::generate(&s.env);
    s.client.register_merchant(&rival, &rival);
    let rival_plan = s
        .client
        .create_plan(&rival, &s.token.address, &PRICE, &MONTH, &0, &0, &DAY);
    assert_eq!(
        s.client.try_change_plan(&s.subscriber, &id, &rival_plan),
        Err(Ok(Error::MerchantMismatch))
    );
}

#[test]
fn switching_to_a_plan_in_another_token_fails() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    let euro = s.env.register_stellar_asset_contract_v2(s.admin.clone());
    let euro_plan =
        s.client
            .create_plan(&s.merchant, &euro.address(), &PRICE, &MONTH, &0, &0, &DAY);
    assert_eq!(
        s.client.try_change_plan(&s.subscriber, &id, &euro_plan),
        Err(Ok(Error::TokenMismatch))
    );
}

#[test]
fn switching_to_a_closed_plan_fails() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    let closed = plan_priced_at(&s, PRICE * 2);
    s.client.set_plan_active(&s.merchant, &closed, &false);
    assert_eq!(
        s.client.try_change_plan(&s.subscriber, &id, &closed),
        Err(Ok(Error::PlanInactive))
    );
}

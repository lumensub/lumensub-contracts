use super::setup::{Setup, DAY, MONTH, PRICE, START};
use crate::{Error, SubscriptionStatus};

#[test]
fn charging_early_is_rejected() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.advance(MONTH - 1);
    assert!(!s.client.is_due(&id));
    assert_eq!(s.client.try_charge(&s.keeper, &id), Err(Ok(Error::NotDue)));
}

#[test]
fn a_due_charge_pays_merchant_treasury_and_keeper() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    let payout_before = s.balance(&s.payout);
    let treasury_before = s.balance(&s.treasury);

    s.advance(MONTH);
    assert!(s.client.is_due(&id));
    assert_eq!(s.client.charge(&s.keeper, &id), PRICE);

    // 1% fee and 0.5% keeper reward come off the top.
    assert_eq!(s.balance(&s.payout) - payout_before, 9_850_000);
    assert_eq!(s.balance(&s.treasury) - treasury_before, 100_000);
    assert_eq!(s.balance(&s.keeper), 50_000);
}

#[test]
fn a_charge_moves_the_schedule_forward_one_period() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.advance(MONTH + 600);
    s.client.charge(&s.keeper, &id);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.cycles_charged, 2);
    assert_eq!(sub.total_paid, PRICE * 2);
    assert_eq!(sub.last_charged_at, START + MONTH + 600);
    // A keeper ten minutes late does not move the billing date.
    assert_eq!(sub.next_charge_at, START + 2 * MONTH);
    assert!(!s.client.is_due(&id));
}

#[test]
fn a_very_late_charge_restarts_the_schedule() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.advance(2 * MONTH + DAY);
    s.client.charge(&s.keeper, &id);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.cycles_charged, 2);
    assert_eq!(sub.next_charge_at, START + 3 * MONTH + DAY);
}

#[test]
fn the_first_charge_after_a_trial_activates_the_subscription() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(7 * DAY, 0, DAY));
    s.advance(7 * DAY);
    assert_eq!(s.client.charge(&s.keeper, &id), PRICE);

    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::Active);
    assert_eq!(sub.cycles_charged, 1);
    assert_eq!(sub.next_charge_at, START + 7 * DAY + MONTH);
}

#[test]
fn unknown_subscriptions_are_never_due() {
    let s = Setup::with_merchant();
    assert!(!s.client.is_due(&77));
    assert_eq!(
        s.client.try_charge(&s.keeper, &77),
        Err(Ok(Error::SubscriptionNotFound))
    );
}

#[test]
fn charging_is_blocked_while_the_protocol_is_paused() {
    let s = Setup::with_merchant();
    let id = s.subscribe(s.create_plan(0, 0, DAY));
    s.advance(MONTH);
    s.client.pause_protocol(&s.admin);
    assert_eq!(
        s.client.try_charge(&s.keeper, &id),
        Err(Ok(Error::ProtocolPaused))
    );
}

#[test]
fn a_limited_plan_completes_after_its_last_cycle() {
    let s = Setup::with_merchant();
    let plan = s.create_plan(0, 3, DAY);
    let id = s.subscribe(plan);
    s.advance(MONTH);
    s.client.charge(&s.keeper, &id);
    assert_eq!(
        s.client.get_subscription(&id).status,
        SubscriptionStatus::Active
    );

    s.advance(MONTH);
    s.client.charge(&s.keeper, &id);
    let sub = s.client.get_subscription(&id);
    assert_eq!(sub.status, SubscriptionStatus::Completed);
    assert_eq!(sub.cycles_charged, 3);
    assert_eq!(s.client.get_plan(&plan).subscriber_count, 0);

    s.advance(MONTH);
    assert!(!s.client.is_due(&id));
    assert_eq!(
        s.client.try_charge(&s.keeper, &id),
        Err(Ok(Error::InvalidStatus))
    );
}

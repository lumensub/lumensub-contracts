//! Subscriber-side actions: signing up, pausing, resuming, cancelling and switching plan.

use lumensub_types::{ChargedEvent, Error, Subscription, SubscriptionStatus};
use soroban_sdk::{Address, Env};

use crate::payments::{self, Shortfall};
use crate::pricing::split_payment;
use crate::{events, governance, storage};

pub(crate) fn subscribe(env: &Env, subscriber: &Address, plan_id: u64) -> Result<u64, Error> {
    subscriber.require_auth();
    let config = governance::require_live(env)?;
    let mut plan = storage::read_plan(env, plan_id)?;
    if !plan.active {
        return Err(Error::PlanInactive);
    }
    let merchant = storage::read_merchant(env, &plan.merchant)?;
    let now = env.ledger().timestamp();

    let id = storage::next_sub_id(env);
    let mut sub = Subscription {
        id,
        plan_id,
        subscriber: subscriber.clone(),
        merchant: plan.merchant.clone(),
        token: plan.token.clone(),
        status: SubscriptionStatus::Trialing,
        created_at: now,
        next_charge_at: now.saturating_add(plan.trial_secs),
        cycles_charged: 0,
        last_charged_at: 0,
        past_due_since: 0,
        paused_at: 0,
        credit: 0,
        total_paid: 0,
        total_refunded: 0,
    };

    // Without a trial the first period is paid up front, so someone who has not approved
    // enough of the token cannot sign up at all.
    let mut first_charge = None;
    if plan.trial_secs == 0 {
        payments::check_can_pay(env, &plan.token, subscriber, plan.amount)
            .map_err(Shortfall::error)?;
        let split = split_payment(plan.amount, config.fee_bps, 0);
        payments::collect(
            env,
            &plan.token,
            subscriber,
            &merchant.payout,
            &config.treasury,
            None,
            &split,
        );
        sub.status = SubscriptionStatus::Active;
        sub.cycles_charged = 1;
        sub.last_charged_at = now;
        sub.total_paid = plan.amount;
        sub.next_charge_at = now.saturating_add(plan.period_secs);
        if plan.max_cycles == 1 {
            sub.status = SubscriptionStatus::Completed;
        }
        first_charge = Some(ChargedEvent {
            amount: plan.amount,
            fee: split.fee,
            keeper_reward: 0,
            cycle: 1,
            next_charge_at: sub.next_charge_at,
        });
    }

    // A one-cycle plan is finished the moment it is paid, so it never holds a seat.
    if sub.status != SubscriptionStatus::Completed {
        plan.subscriber_count += 1;
        storage::write_plan(env, &plan);
    }
    storage::write_subscription(env, &sub);
    events::subscribed(env, id, plan_id, subscriber);
    if let Some(event) = first_charge {
        events::charged(env, id, event);
    }
    Ok(id)
}

fn check_owner(sub: &Subscription, subscriber: &Address) -> Result<(), Error> {
    if sub.subscriber != *subscriber {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

/// Stops billing until the subscriber resumes. Allowed during a protocol pause, so a
/// subscriber can always stop payments.
pub(crate) fn pause(env: &Env, subscriber: &Address, sub_id: u64) -> Result<(), Error> {
    subscriber.require_auth();
    let mut sub = storage::read_subscription(env, sub_id)?;
    check_owner(&sub, subscriber)?;
    if !matches!(
        sub.status,
        SubscriptionStatus::Active | SubscriptionStatus::Trialing
    ) {
        return Err(Error::InvalidStatus);
    }
    sub.status = SubscriptionStatus::Paused;
    sub.paused_at = env.ledger().timestamp();
    storage::write_subscription(env, &sub);
    events::paused(env, sub_id);
    Ok(())
}

pub(crate) fn resume(env: &Env, subscriber: &Address, sub_id: u64) -> Result<(), Error> {
    subscriber.require_auth();
    governance::require_live(env)?;
    let mut sub = storage::read_subscription(env, sub_id)?;
    check_owner(&sub, subscriber)?;
    if sub.status != SubscriptionStatus::Paused {
        return Err(Error::InvalidStatus);
    }
    // Time spent paused is added back, so the subscriber keeps the days they paid for.
    let now = env.ledger().timestamp();
    let paused_for = now.saturating_sub(sub.paused_at);
    sub.next_charge_at = sub.next_charge_at.saturating_add(paused_for);
    sub.paused_at = 0;
    sub.status = if sub.cycles_charged == 0 {
        SubscriptionStatus::Trialing
    } else {
        SubscriptionStatus::Active
    };
    storage::write_subscription(env, &sub);
    events::resumed(env, sub_id);
    Ok(())
}

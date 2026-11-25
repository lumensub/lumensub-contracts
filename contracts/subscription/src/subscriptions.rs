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

    plan.subscriber_count += 1;
    storage::write_plan(env, &plan);
    storage::write_subscription(env, &sub);
    events::subscribed(env, id, plan_id, subscriber);
    if let Some(event) = first_charge {
        events::charged(env, id, event);
    }
    Ok(id)
}

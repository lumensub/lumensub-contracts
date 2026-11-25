//! Keeper-triggered charges. Anyone may call `charge` once a subscription is due; the
//! caller receives the keeper reward. This is what keeps billing running without the
//! merchant or LumenSub having to sign every payment.

use lumensub_types::{ChargedEvent, Error, Subscription, SubscriptionStatus};
use soroban_sdk::{Address, Env};

use crate::payments;
use crate::pricing::{next_charge_time, split_payment};
use crate::{events, governance, plans, storage};

pub(crate) fn charge(env: &Env, keeper: &Address, sub_id: u64) -> Result<i128, Error> {
    keeper.require_auth();
    let config = governance::require_live(env)?;
    let mut sub = storage::read_subscription(env, sub_id)?;
    if !sub.status.is_chargeable() {
        return Err(Error::InvalidStatus);
    }
    let now = env.ledger().timestamp();
    if now < sub.next_charge_at {
        return Err(Error::NotDue);
    }
    let plan = storage::read_plan(env, sub.plan_id)?;
    let merchant = storage::read_merchant(env, &sub.merchant)?;

    let credit_used = sub.credit.min(plan.amount);
    let due = plan.amount - credit_used;
    if due > 0 {
        if let Err(shortfall) = payments::check_can_pay(env, &sub.token, &sub.subscriber, due) {
            record_failure(&mut sub, plan.grace_secs, now);
            if sub.status == SubscriptionStatus::Lapsed {
                plans::release_seat(env, sub.plan_id)?;
            }
            storage::write_subscription(env, &sub);
            events::charge_failed(env, sub_id, shortfall.reason(), sub.status);
            return Ok(0);
        }
    }

    let split = split_payment(due, config.fee_bps, config.keeper_reward_bps);
    payments::collect(
        env,
        &sub.token,
        &sub.subscriber,
        &merchant.payout,
        &config.treasury,
        Some(keeper),
        &split,
    );

    sub.next_charge_at = if sub.status == SubscriptionStatus::PastDue {
        now.saturating_add(plan.period_secs)
    } else {
        next_charge_time(sub.next_charge_at, now, plan.period_secs)
    };
    sub.credit -= credit_used;
    sub.cycles_charged += 1;
    sub.last_charged_at = now;
    sub.total_paid += due;
    sub.past_due_since = 0;
    sub.status = SubscriptionStatus::Active;
    storage::write_subscription(env, &sub);

    let event = ChargedEvent {
        amount: due,
        fee: split.fee,
        keeper_reward: split.keeper,
        cycle: sub.cycles_charged,
        next_charge_at: sub.next_charge_at,
    };
    events::charged(env, sub_id, event);
    Ok(due)
}

/// True when a keeper calling `charge` right now would attempt a payment.
pub(crate) fn is_due(env: &Env, sub_id: u64) -> bool {
    let now = env.ledger().timestamp();
    storage::read_subscription(env, sub_id)
        .is_ok_and(|sub| sub.status.is_chargeable() && now >= sub.next_charge_at)
}

/// Records a failed charge. The first failure moves the subscription to past due and
/// starts the grace period; once the grace period has run out it lapses for good.
fn record_failure(sub: &mut Subscription, grace_secs: u64, now: u64) {
    if sub.status != SubscriptionStatus::PastDue {
        sub.status = SubscriptionStatus::PastDue;
        sub.past_due_since = now;
    }
    if now >= sub.past_due_since.saturating_add(grace_secs) {
        sub.status = SubscriptionStatus::Lapsed;
    }
}

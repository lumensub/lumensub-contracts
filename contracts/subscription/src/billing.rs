//! Keeper-triggered charges. Anyone may call `charge` once a subscription is due; the
//! caller receives the keeper reward. This is what keeps billing running without the
//! merchant or LumenSub having to sign every payment.

use lumensub_types::{ChargedEvent, Error, SubscriptionStatus};
use soroban_sdk::{Address, Env};

use crate::payments::{self, Shortfall};
use crate::pricing::{next_charge_time, split_payment};
use crate::{events, governance, storage};

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
        payments::check_can_pay(env, &sub.token, &sub.subscriber, due)
            .map_err(Shortfall::error)?;
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

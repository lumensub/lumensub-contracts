//! Plans: what a merchant sells, at what price and how often.

use lumensub_types::{Error, Plan, PlanCreatedEvent};
use soroban_sdk::{Address, Env};

use crate::pricing::{MAX_GRACE_SECS, MAX_TRIAL_SECS, MIN_PERIOD_SECS};
use crate::{events, governance, storage};

/// The terms of a new plan, kept together so they are validated in one place.
pub(crate) struct PlanTerms {
    pub token: Address,
    pub amount: i128,
    pub period_secs: u64,
    pub trial_secs: u64,
    pub max_cycles: u32,
    pub grace_secs: u64,
}

fn validate(terms: &PlanTerms) -> Result<(), Error> {
    if terms.amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    if terms.period_secs < MIN_PERIOD_SECS
        || terms.trial_secs > MAX_TRIAL_SECS
        || terms.grace_secs > MAX_GRACE_SECS
    {
        return Err(Error::InvalidPeriod);
    }
    Ok(())
}

pub(crate) fn create(env: &Env, merchant: &Address, terms: PlanTerms) -> Result<u64, Error> {
    merchant.require_auth();
    governance::require_live(env)?;
    let mut record = storage::read_merchant(env, merchant)?;
    validate(&terms)?;

    let id = storage::next_plan_id(env);
    let plan = Plan {
        id,
        merchant: merchant.clone(),
        token: terms.token,
        amount: terms.amount,
        period_secs: terms.period_secs,
        trial_secs: terms.trial_secs,
        max_cycles: terms.max_cycles,
        grace_secs: terms.grace_secs,
        active: true,
        created_at: env.ledger().timestamp(),
        subscriber_count: 0,
    };
    storage::write_plan(env, &plan);

    record.plan_count += 1;
    storage::write_merchant(env, merchant, &record);

    let event = PlanCreatedEvent {
        merchant: merchant.clone(),
        token: plan.token.clone(),
        amount: plan.amount,
        period_secs: plan.period_secs,
    };
    events::plan_created(env, id, event);
    Ok(id)
}

/// Turns new sign-ups on or off. Existing subscribers keep being billed either way, so a
/// merchant can retire a plan without stranding the people already on it.
pub(crate) fn set_active(
    env: &Env,
    merchant: &Address,
    plan_id: u64,
    active: bool,
) -> Result<(), Error> {
    merchant.require_auth();
    let mut plan = storage::read_plan(env, plan_id)?;
    if plan.merchant != *merchant {
        return Err(Error::MerchantMismatch);
    }
    plan.active = active;
    storage::write_plan(env, &plan);
    events::plan_status(env, plan_id, active);
    Ok(())
}

/// Frees a plan's seat when one of its subscriptions ends for good.
pub(crate) fn release_seat(env: &Env, plan_id: u64) -> Result<(), Error> {
    let mut plan = storage::read_plan(env, plan_id)?;
    plan.subscriber_count = plan.subscriber_count.saturating_sub(1);
    storage::write_plan(env, &plan);
    Ok(())
}

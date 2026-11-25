use lumensub_types::{Config, Error, Merchant, Plan, Subscription};
use soroban_sdk::{contracttype, Address, Env};

/// Roughly one day of ledgers at five seconds each.
pub(crate) const DAY_IN_LEDGERS: u32 = 17_280;

const INSTANCE_BUMP: u32 = 7 * DAY_IN_LEDGERS;
const INSTANCE_THRESHOLD: u32 = INSTANCE_BUMP - DAY_IN_LEDGERS;
const PERSISTENT_BUMP: u32 = 30 * DAY_IN_LEDGERS;
const PERSISTENT_THRESHOLD: u32 = PERSISTENT_BUMP - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Config,
    PendingAdmin,
    NextPlanId,
    NextSubId,
    Merchant(Address),
    Plan(u64),
    Sub(u64),
}

pub(crate) fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

pub(crate) fn has_config(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Config)
}

pub(crate) fn read_config(env: &Env) -> Result<Config, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)
}

pub(crate) fn write_config(env: &Env, config: &Config) {
    env.storage().instance().set(&DataKey::Config, config);
    bump_instance(env);
}

pub(crate) fn read_pending_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PendingAdmin)
}

pub(crate) fn write_pending_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::PendingAdmin, admin);
}

pub(crate) fn remove_pending_admin(env: &Env) {
    env.storage().instance().remove(&DataKey::PendingAdmin);
}

fn next_id(env: &Env, key: &DataKey) -> u64 {
    let id = env.storage().instance().get::<_, u64>(key).unwrap_or(0) + 1;
    env.storage().instance().set(key, &id);
    id
}

pub(crate) fn next_plan_id(env: &Env) -> u64 {
    next_id(env, &DataKey::NextPlanId)
}

pub(crate) fn next_sub_id(env: &Env) -> u64 {
    next_id(env, &DataKey::NextSubId)
}

pub(crate) fn plan_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextPlanId)
        .unwrap_or(0)
}

pub(crate) fn subscription_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextSubId)
        .unwrap_or(0)
}

fn bump_persistent(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_THRESHOLD, PERSISTENT_BUMP);
}

pub(crate) fn has_merchant(env: &Env, merchant: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Merchant(merchant.clone()))
}

pub(crate) fn read_merchant(env: &Env, merchant: &Address) -> Result<Merchant, Error> {
    let key = DataKey::Merchant(merchant.clone());
    let record = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::MerchantNotFound)?;
    bump_persistent(env, &key);
    Ok(record)
}

pub(crate) fn write_merchant(env: &Env, merchant: &Address, record: &Merchant) {
    let key = DataKey::Merchant(merchant.clone());
    env.storage().persistent().set(&key, record);
    bump_persistent(env, &key);
}

pub(crate) fn read_plan(env: &Env, plan_id: u64) -> Result<Plan, Error> {
    let key = DataKey::Plan(plan_id);
    let plan = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::PlanNotFound)?;
    bump_persistent(env, &key);
    Ok(plan)
}

pub(crate) fn write_plan(env: &Env, plan: &Plan) {
    let key = DataKey::Plan(plan.id);
    env.storage().persistent().set(&key, plan);
    bump_persistent(env, &key);
}

pub(crate) fn read_subscription(env: &Env, sub_id: u64) -> Result<Subscription, Error> {
    let key = DataKey::Sub(sub_id);
    let sub = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::SubscriptionNotFound)?;
    bump_persistent(env, &key);
    Ok(sub)
}

pub(crate) fn write_subscription(env: &Env, sub: &Subscription) {
    let key = DataKey::Sub(sub.id);
    env.storage().persistent().set(&key, sub);
    bump_persistent(env, &key);
}

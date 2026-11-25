use soroban_sdk::{contracttype, Env};

use crate::types::{Split, SplitError};

const DAY_IN_LEDGERS: u32 = 17_280;
const TTL_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;
const TTL_THRESHOLD: u32 = TTL_EXTEND_TO - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    NextId,
    Split(u64),
}

pub(crate) fn next_id(env: &Env) -> u64 {
    let current: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(0);
    let id = current + 1;
    env.storage().instance().set(&DataKey::NextId, &id);
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
    id
}

pub(crate) fn read_split(env: &Env, id: u64) -> Result<Split, SplitError> {
    let key = DataKey::Split(id);
    let split = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(SplitError::SplitNotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    Ok(split)
}

pub(crate) fn write_split(env: &Env, split: &Split) {
    let key = DataKey::Split(split.id);
    env.storage().persistent().set(&key, split);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

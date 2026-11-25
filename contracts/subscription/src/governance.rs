//! Setup and admin controls: fee rates, the treasury, the protocol pause switch,
//! upgrades and handing the admin role to someone else.

use lumensub_types::{Config, Error};
use soroban_sdk::{Address, BytesN, Env};

use crate::events;
use crate::pricing::{MAX_FEE_BPS, MAX_KEEPER_REWARD_BPS};
use crate::storage;

fn check_rates(fee_bps: u32, keeper_reward_bps: u32) -> Result<(), Error> {
    if fee_bps > MAX_FEE_BPS || keeper_reward_bps > MAX_KEEPER_REWARD_BPS {
        return Err(Error::FeeTooHigh);
    }
    Ok(())
}

pub(crate) fn initialize(
    env: &Env,
    admin: Address,
    treasury: Address,
    fee_bps: u32,
    keeper_reward_bps: u32,
) -> Result<(), Error> {
    if storage::has_config(env) {
        return Err(Error::AlreadyInitialized);
    }
    admin.require_auth();
    check_rates(fee_bps, keeper_reward_bps)?;
    let config = Config {
        admin,
        treasury,
        fee_bps,
        keeper_reward_bps,
        paused: false,
    };
    storage::write_config(env, &config);
    events::config_updated(env, &config);
    Ok(())
}

/// Loads the config and checks that `admin` holds the admin role and signed the call.
pub(crate) fn require_admin(env: &Env, admin: &Address) -> Result<Config, Error> {
    let config = storage::read_config(env)?;
    if config.admin != *admin {
        return Err(Error::Unauthorized);
    }
    admin.require_auth();
    Ok(config)
}

/// Loads the config and fails if the admin has paused the protocol.
pub(crate) fn require_live(env: &Env) -> Result<Config, Error> {
    let config = storage::read_config(env)?;
    if config.paused {
        return Err(Error::ProtocolPaused);
    }
    storage::bump_instance(env);
    Ok(config)
}

pub(crate) fn set_fee(env: &Env, admin: &Address, fee_bps: u32) -> Result<(), Error> {
    let mut config = require_admin(env, admin)?;
    check_rates(fee_bps, config.keeper_reward_bps)?;
    config.fee_bps = fee_bps;
    storage::write_config(env, &config);
    events::config_updated(env, &config);
    Ok(())
}

pub(crate) fn set_keeper_reward(env: &Env, admin: &Address, reward_bps: u32) -> Result<(), Error> {
    let mut config = require_admin(env, admin)?;
    check_rates(config.fee_bps, reward_bps)?;
    config.keeper_reward_bps = reward_bps;
    storage::write_config(env, &config);
    events::config_updated(env, &config);
    Ok(())
}

pub(crate) fn set_treasury(env: &Env, admin: &Address, treasury: Address) -> Result<(), Error> {
    let mut config = require_admin(env, admin)?;
    config.treasury = treasury;
    storage::write_config(env, &config);
    events::config_updated(env, &config);
    Ok(())
}

pub(crate) fn set_paused(env: &Env, admin: &Address, paused: bool) -> Result<(), Error> {
    let mut config = require_admin(env, admin)?;
    config.paused = paused;
    storage::write_config(env, &config);
    events::config_updated(env, &config);
    Ok(())
}

/// First step of an admin handover. The new admin has to accept before anything changes,
/// which protects against handing the contract to a mistyped address.
pub(crate) fn propose_admin(env: &Env, admin: &Address, new_admin: &Address) -> Result<(), Error> {
    require_admin(env, admin)?;
    storage::write_pending_admin(env, new_admin);
    Ok(())
}

pub(crate) fn accept_admin(env: &Env, new_admin: &Address) -> Result<(), Error> {
    let pending = storage::read_pending_admin(env)
        .ok_or(Error::NoPendingAdmin)?;
    if pending != *new_admin {
        return Err(Error::Unauthorized);
    }
    new_admin.require_auth();
    let mut config = storage::read_config(env)?;
    config.admin = new_admin.clone();
    storage::write_config(env, &config);
    storage::remove_pending_admin(env);
    events::config_updated(env, &config);
    Ok(())
}

pub(crate) fn upgrade(env: &Env, admin: &Address, wasm_hash: BytesN<32>) -> Result<(), Error> {
    require_admin(env, admin)?;
    env.deployer().update_current_contract_wasm(wasm_hash);
    Ok(())
}

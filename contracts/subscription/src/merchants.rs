//! Merchant registration. A merchant only needs to register once; registering again
//! updates the payout address.

use lumensub_types::{Error, Merchant};
use soroban_sdk::{Address, Env};

use crate::{events, governance, storage};

pub(crate) fn register(env: &Env, merchant: &Address, payout: &Address) -> Result<(), Error> {
    merchant.require_auth();
    governance::require_live(env)?;
    let record = if storage::has_merchant(env, merchant) {
        let mut existing = storage::read_merchant(env, merchant)?;
        existing.payout = payout.clone();
        existing
    } else {
        Merchant {
            payout: payout.clone(),
            plan_count: 0,
            registered_at: env.ledger().timestamp(),
        }
    };
    storage::write_merchant(env, merchant, &record);
    events::merchant_registered(env, merchant, payout);
    Ok(())
}

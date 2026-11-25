//! Merchant-initiated refunds. Refunds come from the merchant's own balance, not from
//! the treasury or the keeper, and can never exceed what the subscriber has paid.

use lumensub_types::Error;
use soroban_sdk::{token, Address, Env};

use crate::{events, storage};

pub(crate) fn refund(
    env: &Env,
    merchant: &Address,
    sub_id: u64,
    amount: i128,
) -> Result<(), Error> {
    merchant.require_auth();
    let mut sub = storage::read_subscription(env, sub_id)?;
    if sub.merchant != *merchant {
        return Err(Error::MerchantMismatch);
    }
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    let refundable = sub.total_paid - sub.total_refunded;
    if amount > refundable {
        return Err(Error::RefundTooLarge);
    }

    let client = token::Client::new(env, &sub.token);
    client.transfer(merchant, &sub.subscriber, &amount);
    sub.total_refunded += amount;
    storage::write_subscription(env, &sub);
    events::refunded(env, sub_id, amount);
    Ok(())
}

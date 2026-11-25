//! Moving tokens. The contract never holds subscriber funds: every charge is pulled
//! straight from the subscriber to the merchant, the treasury and the keeper using the
//! allowance the subscriber granted.

use lumensub_types::Error;
use soroban_sdk::{symbol_short, token, Address, Env, Symbol};

use crate::pricing::PaymentSplit;

/// Why a subscriber cannot be charged right now.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Shortfall {
    Allowance,
    Balance,
}

impl Shortfall {
    /// Short reason published in the `chg_fail` event.
    pub(crate) fn reason(self) -> Symbol {
        match self {
            Shortfall::Allowance => symbol_short!("allowance"),
            Shortfall::Balance => symbol_short!("balance"),
        }
    }

    /// Error returned when the shortfall blocks a call outright, as in `subscribe`.
    pub(crate) fn error(self) -> Error {
        match self {
            Shortfall::Allowance => Error::InsufficientAllowance,
            Shortfall::Balance => Error::InsufficientBalance,
        }
    }
}

/// Checks that `from` has approved this contract for `amount` and holds at least that
/// much. Checking first lets a failed charge be recorded instead of aborting the call.
pub(crate) fn check_can_pay(
    env: &Env,
    token: &Address,
    from: &Address,
    amount: i128,
) -> Result<(), Shortfall> {
    let client = token::Client::new(env, token);
    let spender = env.current_contract_address();
    if client.allowance(from, &spender) < amount {
        return Err(Shortfall::Allowance);
    }
    if client.balance(from) < amount {
        return Err(Shortfall::Balance);
    }
    Ok(())
}

/// Pulls one charge from `from` and sends each share where it belongs. Zero shares are
/// skipped so a protocol with no fee makes no empty transfers. `keeper` is `None` when
/// the subscriber pays directly, as on sign-up.
pub(crate) fn collect(
    env: &Env,
    token: &Address,
    from: &Address,
    payout: &Address,
    treasury: &Address,
    keeper: Option<&Address>,
    split: &PaymentSplit,
) {
    let client = token::Client::new(env, token);
    let spender = env.current_contract_address();
    if split.merchant > 0 {
        client.transfer_from(&spender, from, payout, &split.merchant);
    }
    if split.fee > 0 {
        client.transfer_from(&spender, from, treasury, &split.fee);
    }
    if let Some(keeper) = keeper {
        if split.keeper > 0 {
            client.transfer_from(&spender, from, keeper, &split.keeper);
        }
    }
}

use soroban_sdk::{Address, Vec};

use crate::types::SplitError;
use crate::{MAX_RECIPIENTS, TOTAL_BPS};

/// Checks that a recipient list is non-empty, has at most ten distinct addresses, gives
/// each a non-zero share, and adds up to exactly 10,000 basis points.
pub(crate) fn check_recipients(recipients: &Vec<(Address, u32)>) -> Result<(), SplitError> {
    let count = recipients.len();
    if count == 0 {
        return Err(SplitError::NoRecipients);
    }
    if count > MAX_RECIPIENTS {
        return Err(SplitError::TooManyRecipients);
    }

    let mut total: u32 = 0;
    for (index, (address, bps)) in recipients.iter().enumerate() {
        if bps == 0 {
            return Err(SplitError::InvalidShares);
        }
        let seen_before = recipients
            .iter()
            .take(index)
            .any(|(other, _)| other == address);
        if seen_before {
            return Err(SplitError::DuplicateRecipient);
        }
        total = total.checked_add(bps).ok_or(SplitError::InvalidShares)?;
    }
    if total != TOTAL_BPS {
        return Err(SplitError::InvalidShares);
    }
    Ok(())
}

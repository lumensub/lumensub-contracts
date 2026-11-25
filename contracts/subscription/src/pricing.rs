//! Pure arithmetic for fees, proration and scheduling. No storage access here, so it
//! can be tested without an environment.

pub(crate) const BPS_DENOMINATOR: i128 = 10_000;
/// Highest protocol fee the admin can set: 10%.
pub(crate) const MAX_FEE_BPS: u32 = 1_000;
/// Highest keeper reward the admin can set: 5%.
pub(crate) const MAX_KEEPER_REWARD_BPS: u32 = 500;
/// Shortest billing period: one hour.
pub(crate) const MIN_PERIOD_SECS: u64 = 3_600;
/// Longest free trial: one year.
pub(crate) const MAX_TRIAL_SECS: u64 = 365 * 86_400;
/// Longest grace period after a failed charge: thirty days.
pub(crate) const MAX_GRACE_SECS: u64 = 30 * 86_400;

/// How one charge is divided between the merchant, the treasury and the keeper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaymentSplit {
    pub merchant: i128,
    pub fee: i128,
    pub keeper: i128,
}

/// Splits `amount` into the protocol fee, the keeper reward and what is left for the
/// merchant. Fees round down, so any dust goes to the merchant.
pub(crate) fn split_payment(amount: i128, fee_bps: u32, keeper_bps: u32) -> PaymentSplit {
    let fee = amount * i128::from(fee_bps) / BPS_DENOMINATOR;
    let keeper = amount * i128::from(keeper_bps) / BPS_DENOMINATOR;
    PaymentSplit {
        merchant: amount - fee - keeper,
        fee,
        keeper,
    }
}

/// Value of the unused part of a period, used as credit when a subscriber changes plan.
pub(crate) fn prorated_credit(amount: i128, period_secs: u64, remaining_secs: u64) -> i128 {
    if period_secs == 0 {
        return 0;
    }
    let remaining = remaining_secs.min(period_secs);
    amount * i128::from(remaining) / i128::from(period_secs)
}

/// When the next charge should happen after a successful one.
///
/// Normally the schedule simply moves forward one period, so a keeper that is a few
/// minutes late does not shift the subscriber's billing date. If the charge is a whole
/// period or more late, the schedule restarts from now instead, which avoids charging
/// the subscriber twice in quick succession.
pub(crate) fn next_charge_time(scheduled: u64, now: u64, period_secs: u64) -> u64 {
    let on_schedule = scheduled.saturating_add(period_secs);
    if now >= on_schedule {
        now.saturating_add(period_secs)
    } else {
        on_schedule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_takes_fee_and_reward_off_the_top() {
        let split = split_payment(10_000_000, 100, 50);
        assert_eq!(split.fee, 100_000);
        assert_eq!(split.keeper, 50_000);
        assert_eq!(split.merchant, 9_850_000);
    }

    #[test]
    fn split_rounds_fees_down() {
        let split = split_payment(999, 100, 50);
        assert_eq!(split.fee, 9);
        assert_eq!(split.keeper, 4);
        assert_eq!(split.merchant, 986);
        assert_eq!(split.merchant + split.fee + split.keeper, 999);
    }

    #[test]
    fn split_with_no_fees_pays_merchant_everything() {
        let split = split_payment(500, 0, 0);
        assert_eq!(split.merchant, 500);
        assert_eq!(split.fee, 0);
        assert_eq!(split.keeper, 0);
    }

    #[test]
    fn credit_is_proportional_to_time_left() {
        assert_eq!(prorated_credit(3_000, 30, 10), 1_000);
        assert_eq!(prorated_credit(3_000, 30, 0), 0);
    }

    #[test]
    fn credit_never_exceeds_one_period() {
        assert_eq!(prorated_credit(3_000, 30, 90), 3_000);
        assert_eq!(prorated_credit(3_000, 0, 10), 0);
    }

    #[test]
    fn schedule_moves_forward_one_period_when_on_time() {
        assert_eq!(next_charge_time(1_000, 1_000, 100), 1_100);
        assert_eq!(next_charge_time(1_000, 1_050, 100), 1_100);
    }

    #[test]
    fn schedule_restarts_when_a_whole_period_late() {
        assert_eq!(next_charge_time(1_000, 1_100, 100), 1_200);
        assert_eq!(next_charge_time(1_000, 1_500, 100), 1_600);
    }
}

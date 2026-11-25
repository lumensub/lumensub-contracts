use soroban_sdk::contracttype;

/// Where a subscription is in its life.
///
/// Encoded on chain as a `u32`, so indexers can store it as a plain integer.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SubscriptionStatus {
    /// Inside the free trial. The first charge happens when the trial ends.
    Trialing = 0,
    /// Paid up. The next charge happens at `next_charge_at`.
    Active = 1,
    /// The last charge failed. Keepers keep retrying until the grace period ends.
    PastDue = 2,
    /// Paused by the subscriber. No charges happen while paused.
    Paused = 3,
    /// Cancelled by the subscriber or the merchant. Final.
    Cancelled = 4,
    /// The grace period ran out without a successful charge. Final.
    Lapsed = 5,
    /// The plan's `max_cycles` have all been charged. Final.
    Completed = 6,
}

impl SubscriptionStatus {
    /// Statuses in which a keeper may attempt a charge.
    pub fn is_chargeable(self) -> bool {
        matches!(
            self,
            SubscriptionStatus::Trialing | SubscriptionStatus::Active | SubscriptionStatus::PastDue
        )
    }

    /// Statuses that can never change again.
    pub fn is_final(self) -> bool {
        matches!(
            self,
            SubscriptionStatus::Cancelled
                | SubscriptionStatus::Lapsed
                | SubscriptionStatus::Completed
        )
    }
}

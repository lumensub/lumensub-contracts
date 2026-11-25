use soroban_sdk::contracterror;

/// Errors returned by the subscription contract.
///
/// The numeric codes are part of the public interface and never change.
/// New errors are added at the end.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// `initialize` has not been called yet.
    NotInitialized = 1,
    /// `initialize` was called on a contract that is already set up.
    AlreadyInitialized = 2,
    /// The caller is not allowed to perform this action.
    Unauthorized = 3,
    /// No merchant is registered under this address.
    MerchantNotFound = 4,
    /// No plan exists with this id.
    PlanNotFound = 5,
    /// The plan has been deactivated and accepts no new subscribers.
    PlanInactive = 6,
    /// No subscription exists with this id.
    SubscriptionNotFound = 7,
    /// The subscription is not due for a charge yet.
    NotDue = 8,
    /// The subscription is in a status that does not allow this action.
    InvalidStatus = 9,
    /// An amount was zero, negative or otherwise out of range.
    InvalidAmount = 10,
    /// A period, trial or grace duration was out of range.
    InvalidPeriod = 11,
    /// A fee or keeper reward exceeds the allowed maximum.
    FeeTooHigh = 12,
    /// The protocol is paused by the admin.
    ProtocolPaused = 13,
    /// The two plans are priced in different tokens.
    TokenMismatch = 14,
    /// The plan or subscription belongs to a different merchant.
    MerchantMismatch = 15,
    /// The refund would exceed what the subscriber has paid.
    RefundTooLarge = 16,
    /// The subscriber has not approved enough of the token for this contract.
    InsufficientAllowance = 17,
    /// The subscriber's token balance is too low.
    InsufficientBalance = 18,
    /// `accept_admin` was called with no transfer in progress.
    NoPendingAdmin = 19,
    /// `change_plan` was asked to switch to the plan already in use.
    SamePlan = 20,
}

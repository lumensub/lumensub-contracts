use soroban_sdk::{contracterror, contracttype, Address, Vec};

/// Errors returned by the splitter. Codes are stable and only ever appended.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SplitError {
    /// No split exists with this id.
    SplitNotFound = 1,
    /// A share is zero or the shares do not add up to 10,000.
    InvalidShares = 2,
    /// More than ten recipients.
    TooManyRecipients = 3,
    /// The amount to distribute is zero or negative.
    InvalidAmount = 4,
    /// The same address appears twice.
    DuplicateRecipient = 5,
    /// The caller does not own this split.
    Unauthorized = 6,
    /// The recipient list is empty.
    NoRecipients = 7,
}

/// A fixed division of incoming revenue.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Split {
    pub id: u64,
    /// The wallet funds are pulled from, and the only account that can edit the split.
    pub owner: Address,
    /// Each recipient with their share in basis points.
    pub recipients: Vec<(Address, u32)>,
}

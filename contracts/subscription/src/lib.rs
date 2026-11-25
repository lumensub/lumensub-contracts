//! The LumenSub subscription contract.
//!
//! Merchants register and publish plans. Subscribers approve the contract to spend a
//! token on their behalf and subscribe to a plan. From then on anyone can call `charge`
//! once a payment is due; the caller (a "keeper") earns a small reward for doing so.
#![no_std]

mod storage;

pub use lumensub_types::*;

use soroban_sdk::contract;

#[contract]
pub struct SubscriptionContract;

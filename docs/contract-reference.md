# Contract reference

Everything the contracts expose. Function names, argument order, event topics and error
codes are a stable interface; the backend and the app are built against this page.

Amounts are `i128` in the token's smallest unit (stroops for a 7-decimal Stellar asset).
Times are ledger timestamps in seconds. Basis points (bps) are hundredths of a percent.

## Subscription contract

### Setup and administration

| Function | Who | What it does |
|---|---|---|
| `initialize(admin, treasury, fee_bps, keeper_reward_bps)` | admin | One-time setup. `fee_bps` at most 1000, `keeper_reward_bps` at most 500. |
| `get_config() -> Config` | anyone | Admin, treasury, rates and whether the protocol is paused. |
| `set_fee(admin, fee_bps)` | admin | Changes the protocol fee. |
| `set_keeper_reward(admin, reward_bps)` | admin | Changes the keeper reward. |
| `set_treasury(admin, treasury)` | admin | Changes where protocol fees go. |
| `pause_protocol(admin)` | admin | Blocks new merchants, plans, subscriptions, charges, resumes and plan changes. |
| `unpause_protocol(admin)` | admin | Lifts the pause. |
| `propose_admin(admin, new_admin)` | admin | Starts an admin handover. |
| `accept_admin(new_admin)` | proposed admin | Completes the handover. |
| `upgrade(admin, wasm_hash)` | admin | Replaces the contract code. Storage is kept. |

### Merchants

| Function | Who | What it does |
|---|---|---|
| `register_merchant(merchant, payout)` | merchant | Registers, or updates the payout address if already registered. |
| `get_merchant(merchant) -> Merchant` | anyone | Payout address, plan count and registration time. |

### Plans

| Function | Who | What it does |
|---|---|---|
| `create_plan(merchant, token, amount, period_secs, trial_secs, max_cycles, grace_secs) -> u64` | merchant | Publishes a plan and returns its id. |
| `set_plan_active(merchant, plan_id, active)` | merchant | Opens or closes the plan to new subscribers. Existing subscribers keep being billed. |
| `get_plan(plan_id) -> Plan` | anyone | |
| `plan_count() -> u64` | anyone | Plan ids run from 1 to this number. |

Limits on `create_plan`: `amount` above zero, `period_secs` at least 3,600 (one hour),
`trial_secs` at most one year, `grace_secs` at most thirty days. `max_cycles = 0` means
no limit.

### Subscriptions

| Function | Who | What it does |
|---|---|---|
| `subscribe(subscriber, plan_id) -> u64` | subscriber | Subscribes and returns the id. Charges the first period unless the plan has a trial. |
| `get_subscription(sub_id) -> Subscription` | anyone | |
| `subscription_count() -> u64` | anyone | Subscription ids run from 1 to this number. |
| `pause(subscriber, sub_id)` | subscriber | Stops billing. Works during a protocol pause. |
| `resume(subscriber, sub_id)` | subscriber | Restarts billing and adds the paused time to `next_charge_at`. |
| `cancel(caller, sub_id)` | subscriber or merchant | Ends the subscription. Works during a protocol pause. |
| `change_plan(subscriber, sub_id, new_plan_id)` | subscriber | Moves to another plan from the same merchant in the same token, with prorated credit. |

Before `subscribe`, the subscriber calls `approve(subscriber, <subscription contract>,
amount, expiration_ledger)` on the token. The allowance has to cover every charge you
expect before `expiration_ledger`, so approve for months, not one period.

### Billing

| Function | Who | What it does |
|---|---|---|
| `charge(keeper, sub_id) -> i128` | anyone | Charges a due subscription and returns the amount taken, or zero if the charge failed. |
| `is_due(sub_id) -> bool` | anyone | True when `charge` would attempt a payment now. |

A successful charge pulls the amount due (plan price minus any credit) from the
subscriber and sends:

- `amount * fee_bps / 10000` to the treasury,
- `amount * keeper_reward_bps / 10000` to the keeper,
- the rest to the merchant's payout address.

Fees round down, so rounding dust goes to the merchant.

### Refunds

| Function | Who | What it does |
|---|---|---|
| `refund(merchant, sub_id, amount)` | merchant | Sends `amount` from the merchant's own wallet back to the subscriber. Capped at total paid minus earlier refunds. |

### Stored records

`Config`: `admin`, `treasury`, `fee_bps`, `keeper_reward_bps`, `paused`.

`Merchant`: `payout`, `plan_count`, `registered_at`.

`Plan`: `id`, `merchant`, `token`, `amount`, `period_secs`, `trial_secs`, `max_cycles`,
`grace_secs`, `active`, `created_at`, `subscriber_count`.

`Subscription`: `id`, `plan_id`, `subscriber`, `merchant`, `token`, `status`,
`created_at`, `next_charge_at`, `cycles_charged`, `last_charged_at`, `past_due_since`,
`paused_at`, `credit`, `total_paid`, `total_refunded`.

`status` is a `u32`; the values are listed in [lifecycle.md](lifecycle.md).

### Events

The first topic is a short symbol, the second is the id it concerns. Data structs are
defined in `contracts/lumensub-types/src/events.rs`.

| Topics | Data | When |
|---|---|---|
| `("config")` | `Config` | Any change to the config, including initialization. |
| `("merchant", merchant)` | payout `Address` | A merchant registers or changes payout. |
| `("plan_new", plan_id)` | `{ merchant, token, amount, period_secs }` | A plan is created. |
| `("plan_act", plan_id)` | `bool` | A plan is opened or closed. |
| `("sub_new", sub_id)` | `{ plan_id, subscriber }` | Someone subscribes. |
| `("charged", sub_id)` | `{ amount, fee, keeper_reward, cycle, next_charge_at }` | A charge succeeds, including the first one on sign-up. |
| `("chg_fail", sub_id)` | `{ reason, status }` | A charge fails. `reason` is `allowance` or `balance`; `status` is `PastDue` or `Lapsed`. |
| `("paused", sub_id)` | none | A subscription is paused. |
| `("resumed", sub_id)` | none | A subscription is resumed. |
| `("cancelled", sub_id)` | `{ by }` | A subscription is cancelled. `by` is the subscriber or the merchant. |
| `("plan_chg", sub_id)` | `{ from_plan, to_plan }` | A subscriber changes plan. |
| `("refunded", sub_id)` | `{ amount }` | A merchant refunds. |

On sign-up without a trial, `sub_new` is published first and `charged` (with
`cycle: 1` and `keeper_reward: 0`) right after it.

### Errors

| Code | Name | Meaning |
|---|---|---|
| 1 | `NotInitialized` | `initialize` has not been called. |
| 2 | `AlreadyInitialized` | `initialize` was called twice. |
| 3 | `Unauthorized` | The caller is not allowed to do this. |
| 4 | `MerchantNotFound` | No merchant at this address. |
| 5 | `PlanNotFound` | No plan with this id. |
| 6 | `PlanInactive` | The plan is closed to new subscribers. |
| 7 | `SubscriptionNotFound` | No subscription with this id. |
| 8 | `NotDue` | The subscription is not due yet. |
| 9 | `InvalidStatus` | The subscription's status does not allow this. |
| 10 | `InvalidAmount` | Zero, negative or out-of-range amount. |
| 11 | `InvalidPeriod` | Period, trial or grace out of range. |
| 12 | `FeeTooHigh` | Fee or keeper reward above the cap. |
| 13 | `ProtocolPaused` | The admin has paused the protocol. |
| 14 | `TokenMismatch` | The plans use different tokens. |
| 15 | `MerchantMismatch` | The plan or subscription belongs to another merchant. |
| 16 | `RefundTooLarge` | The refund exceeds what is refundable. |
| 17 | `InsufficientAllowance` | The subscriber has not approved enough for this contract. |
| 18 | `InsufficientBalance` | The subscriber's balance is too low. |
| 19 | `NoPendingAdmin` | `accept_admin` with no handover in progress. |
| 20 | `SamePlan` | `change_plan` to the plan already in use. |

Codes never change and new ones are only appended.

## Splitter contract

A split pays incoming revenue out to several wallets by fixed shares. It is separate
from the subscription contract and entirely optional.

The split's owner approves the splitter to spend the token. `distribute` then pulls an
amount from the owner and pays each recipient their share. A merchant who wants revenue
shared sets the owner wallet as their LumenSub payout and has a keeper call
`distribute` after each `charged` event.

| Function | Who | What it does |
|---|---|---|
| `create_split(owner, recipients) -> u64` | owner | `recipients` is a list of `(Address, bps)` pairs adding up to 10,000. At most ten, no duplicates, no zero shares. |
| `update_split(owner, split_id, recipients)` | owner | Replaces the list, with the same checks. |
| `distribute(split_id, token, amount)` | anyone | Pulls `amount` from the owner and pays out. Shares round down and the last recipient gets the remainder. |
| `get_split(split_id) -> Split` | anyone | |

Events: `("split_new", split_id)` with the owner, `("split_upd", split_id)` with the
owner, and `("split_out", split_id)` with `(token, amount)`.

| Code | Name | Meaning |
|---|---|---|
| 1 | `SplitNotFound` | No split with this id. |
| 2 | `InvalidShares` | A zero share, or shares not adding up to 10,000. |
| 3 | `TooManyRecipients` | More than ten recipients. |
| 4 | `InvalidAmount` | Zero or negative amount. |
| 5 | `DuplicateRecipient` | An address appears twice. |
| 6 | `Unauthorized` | Not the split's owner. |
| 7 | `NoRecipients` | Empty recipient list. |

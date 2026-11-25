# Architecture

LumenSub is three pieces: these contracts, a backend that watches them and triggers
charges, and an app for merchants and subscribers. The contracts are the source of
truth. Everything else can be rebuilt from their storage and events.

## Contracts

| Crate | What it is |
|---|---|
| `contracts/subscription` | The core contract: merchants, plans, subscriptions, charges, refunds and admin controls. |
| `contracts/splitter` | An optional contract that splits a payout between several wallets by fixed shares. |
| `contracts/lumensub-types` | Shared types, events and error codes. Not deployed on its own. |

## The pull model

Card networks let a merchant pull money on a schedule. Stellar payments are pushed by
the sender, which is great for one-off payments and awkward for subscriptions. Soroban
token allowances bridge the gap.

1. The subscriber calls `approve` on the token contract, letting the subscription
   contract spend up to some amount until some ledger.
2. The subscriber calls `subscribe`. Without a trial, the first period is pulled
   straight away.
3. When the next payment is due, anyone may call `charge`. The contract pulls the plan
   price from the subscriber and splits it between the merchant's payout address, the
   protocol treasury and the caller.

The contract never holds subscriber funds. Each charge moves tokens directly from the
subscriber to the three recipients in the same transaction.

The subscriber stays in control the whole time. Lowering the allowance to zero stops
every future charge, even without touching LumenSub. Pausing and cancelling are
available as contract calls too, and both work while the protocol is paused.

## Keepers

Nothing on Stellar runs on a timer, so somebody has to call `charge`. We call that
somebody a keeper. The LumenSub backend runs one, but the function is open to anyone,
and the keeper reward (a small cut of each charge, 0.5% by default) pays for the
transaction fee with room to spare. If our backend went away, any third party could
keep billing running.

A keeper cannot charge early, charge twice for the same period, or charge more than the
plan price. The worst a keeper can do is not show up, and then the charge happens late.
See [keepers.md](keepers.md).

## Storage

| Data | Storage | Key |
|---|---|---|
| Config, id counters, pending admin | instance | singletons |
| Merchants | persistent | merchant address |
| Plans | persistent | plan id |
| Subscriptions | persistent | subscription id |

Every read or write of a persistent entry extends its TTL to about thirty days, so
anything in active use stays live. A subscription that nobody touches for a month can
be archived by the network and restored later with a restore operation; its data is not
lost.

## Events

Every state change publishes an event whose second topic is the plan or subscription
id. The backend indexer builds its database from these alone. The exact topics and
payloads are listed in [contract-reference.md](contract-reference.md) and pinned by tests
in `contracts/subscription/src/test/events.rs`.

## Upgrades

The admin can replace the contract code with `upgrade`. Storage layout is kept, so a new
version must read the old records as they are. Adding fields to a stored struct is a
breaking change and needs a migration plan.

# LumenSub contracts

[![CI](https://github.com/lumensub/lumensub-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/lumensub/lumensub-contracts/actions/workflows/ci.yml)

Soroban smart contracts for recurring payments on Stellar.

LumenSub lets a business charge customers on a schedule, in USDC or any other Stellar
asset, without holding their money and without a payment processor in the middle. A
merchant publishes a plan, a customer subscribes once, and from then on each payment
happens when it is due. The customer can pause or cancel at any time, and can always
cut LumenSub off entirely from their own wallet.

This repository is the on-chain part. The [backend](https://github.com/lumensub/lumensub-backend)
indexes the contracts and runs the billing scheduler, and the [app](https://github.com/lumensub/lumensub-app)
is the merchant dashboard, checkout page and subscriber portal.

## Why this exists

Stellar is good at moving money cheaply and quickly, but every payment is pushed by
the person paying. That works for invoices and transfers. It does not work for a gym
membership, a SaaS seat or a newsletter, where nobody wants to sign a transaction every
month and the business needs to know payment will arrive.

Soroban token allowances make a pull model possible. The subscriber approves the
LumenSub contract to spend up to a limit, once. The contract can then take each period's
payment when it falls due, and nothing more. The rules (how much, how often, for how
long) live in the contract where both sides can read them.

## How a payment happens

1. The subscriber approves the subscription contract on the token, for as many
   periods as they like.
2. They call `subscribe`. Unless the plan has a free trial, the first period is paid
   right away.
3. When the next payment is due, anyone can call `charge`. The contract pulls the plan
   price from the subscriber and sends it, in the same transaction, to the merchant's
   payout address, the protocol treasury (1% by default) and whoever made the call
   (0.5% by default).

That last part matters. Because the caller earns a small reward, billing does not
depend on LumenSub running a server. Our backend triggers charges, but so can anyone
else, and a charge can never be early, doubled or larger than the plan price.

If a charge fails because the allowance or balance is too low, the subscription goes
past due and keepers retry during the plan's grace period. If it is still unpaid when
the grace period ends, it lapses.

## What's in the box

- **Plans** with a price, a billing period, an optional free trial, an optional limit
  on the number of payments, and a grace period for failed payments.
- **Subscriptions** that can be paused (the paused time is added back), resumed,
  cancelled by either side, and moved to another plan with prorated credit.
- **Refunds** from the merchant, capped at what the subscriber has actually paid.
- **A revenue splitter** for merchants who share income with partners or affiliates.
- **Admin controls**: fee and keeper-reward caps, a protocol pause that still lets
  subscribers cancel, two-step admin handover and code upgrades.

## Layout

```
contracts/
  subscription/     the core contract
  splitter/         optional revenue splits
  lumensub-types/   shared records, events and error codes
docs/
  architecture.md       how the pieces fit together
  lifecycle.md          subscription statuses and timing rules
  contract-reference.md every function, event and error
  integration.md        a full walkthrough with the Stellar CLI
  keepers.md            running your own keeper
scripts/
  deploy-testnet.sh     build, deploy and initialize on testnet
```

## Getting started

You need Rust (stable) with the `wasm32v1-none` target. With rustup, the toolchain file
in this repo installs both on first use.

```sh
git clone https://github.com/lumensub/lumensub-contracts.git
cd lumensub-contracts
make test     # unit and integration tests
make check    # what CI runs: fmt, clippy, tests, release wasm build
```

To deploy your own copy to testnet, install the
[Stellar CLI](https://developers.stellar.org/docs/tools/cli) and run:

```sh
./scripts/deploy-testnet.sh
```

It creates and funds an admin identity if needed, deploys both contracts, initializes
the subscription contract in the same run, and writes the contract ids to
`.env.testnet`. [docs/integration.md](docs/integration.md) picks up from there.

## Status

Early and on testnet only. The contracts have not been audited, and the interface may
still change before a 1.0 release. Please do not use them with real funds yet. See
[SECURITY.md](SECURITY.md) for how to report a problem.

## Contributing

Issues and pull requests are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) explains how
we work and what a good pull request looks like here. If you are new to Soroban, the
issues labelled `good first issue` are a gentle place to start.

## License

[MIT](LICENSE)

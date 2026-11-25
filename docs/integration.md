# Integrating with LumenSub

This page walks through a full subscription from the command line with the
[Stellar CLI](https://developers.stellar.org/docs/tools/cli), so you can see every call
the app and backend make. It assumes you have deployed the contracts with
`scripts/deploy-testnet.sh` and have its output in your shell.

```sh
export SUBSCRIPTION=<subscription contract id>
export TOKEN=<token contract id, e.g. testnet USDC>
```

## 1. Merchant: register and create a plan

```sh
stellar contract invoke --id $SUBSCRIPTION --source merchant --network testnet -- \
  register_merchant --merchant merchant --payout merchant

# 5 USDC (7 decimals) every 30 days, 7-day trial, no cycle limit, 3-day grace.
stellar contract invoke --id $SUBSCRIPTION --source merchant --network testnet -- \
  create_plan --merchant merchant --token $TOKEN \
  --amount 50000000 --period_secs 2592000 --trial_secs 604800 \
  --max_cycles 0 --grace_secs 259200
```

The second call prints the plan id.

## 2. Subscriber: approve, then subscribe

The allowance is what lets the contract charge you later. Approve enough for as many
periods as you are comfortable with, and pick an expiration ledger far enough out to
cover them. Ledgers close about every five seconds, so 535,000 ledgers is roughly a
month; the network caps how far ahead an allowance can be set.

```sh
CURRENT=$(stellar ledger latest --network testnet --output json | jq .sequence)

stellar contract invoke --id $TOKEN --source alice --network testnet -- \
  approve --from alice --spender $SUBSCRIPTION \
  --amount 600000000 --expiration_ledger $((CURRENT + 3000000))

stellar contract invoke --id $SUBSCRIPTION --source alice --network testnet -- \
  subscribe --subscriber alice --plan_id 1
```

When an allowance is about to expire, the subscriber simply approves again. Nothing in
LumenSub needs to change.

## 3. Anyone: charge when due

```sh
stellar contract invoke --id $SUBSCRIPTION --source keeper --network testnet -- \
  is_due --sub_id 1

stellar contract invoke --id $SUBSCRIPTION --source keeper --network testnet -- \
  charge --keeper keeper --sub_id 1
```

## 4. Subscriber: pause, resume, switch or cancel

```sh
stellar contract invoke --id $SUBSCRIPTION --source alice --network testnet -- \
  pause --subscriber alice --sub_id 1

stellar contract invoke --id $SUBSCRIPTION --source alice --network testnet -- \
  resume --subscriber alice --sub_id 1

stellar contract invoke --id $SUBSCRIPTION --source alice --network testnet -- \
  change_plan --subscriber alice --sub_id 1 --new_plan_id 2

stellar contract invoke --id $SUBSCRIPTION --source alice --network testnet -- \
  cancel --caller alice --sub_id 1
```

## TypeScript bindings

To call the contract from a web app or a Node service, generate typed bindings:

```sh
stellar contract bindings typescript --network testnet \
  --contract-id $SUBSCRIPTION --output-dir bindings/subscription
```

The generated client exposes every function on this page with typed arguments and
results, including the error enum.

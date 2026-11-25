# Security

LumenSub moves real money once it leaves testnet, so we take reports seriously.

## Status

The contracts have **not been audited**. They are deployed on Stellar testnet for
development and should not hold mainnet funds until an audit is done and published here.

## Reporting a vulnerability

Please report privately through GitHub's
[security advisory form](https://github.com/lumensub/lumensub-contracts/security/advisories/new).
Don't open a public issue or pull request for anything that could let someone take or
lock funds.

A useful report says which function is affected, what an attacker can do, and how to
reproduce it. A failing test is ideal. We aim to reply within three working days and to
keep you updated until a fix ships.

## Known design choices

These are deliberate. Reports that only restate them will be closed, but we are glad to
discuss better designs in an issue.

- **Initialization is open until it happens.** Deploy and call `initialize` in the same
  transaction, or use `scripts/deploy-testnet.sh`, which does both back to back.
- **Charges pull from an allowance.** A subscriber can stop all future charges at any
  time by setting their allowance to zero on the token contract, or by pausing or
  cancelling. The contract never holds subscriber funds between charges.
- **Anyone can call `charge`.** That is how billing runs without a central server. A
  charge can only happen when the subscription is due, and only for the plan price.
- **The admin can upgrade the contract.** Until governance moves to a multisig with a
  timelock, the admin key is a single point of trust.

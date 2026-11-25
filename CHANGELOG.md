# Changelog

All notable changes to the contracts are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org). Until 1.0, minor versions may change the
contract interface; any such change is called out under **Interface**.

## [Unreleased]

### Added

- Subscription contract with merchants, plans, subscriptions, keeper-triggered charges,
  past-due handling with grace periods, pausing, cancelling, plan changes with
  prorated credit, and merchant refunds.
- Admin controls: fee and keeper-reward caps, protocol pause, two-step admin handover
  and code upgrades.
- Splitter contract for sharing payouts between up to ten recipients.
- Shared `lumensub-types` crate with records, event payloads and stable error codes.
- Testnet deploy script, CI, and documentation for the architecture, lifecycle,
  contract reference, keepers and integration.

# Contributing

Thanks for wanting to help. This repository holds the on-chain half of LumenSub, so
changes here are reviewed slowly and carefully. That is not a reflection on you; a bug
in a payments contract is expensive in a way most bugs are not.

## Before you start

- Look for an open issue first. If there isn't one, open it and describe what you want
  to change before writing much code. For anything touching how money moves, wait for a
  maintainer to agree on the approach.
- Small pull requests get merged. Large ones sit. If a change naturally splits in two,
  split it.

## Setting up

You need Rust (stable) and the `wasm32v1-none` target. The toolchain file in the repo
root takes care of both if you use rustup.

```sh
make test     # run all tests
make check    # what CI runs: fmt, clippy, tests and a release wasm build
```

The [Stellar CLI](https://developers.stellar.org/docs/tools/cli) is only needed if you
want to deploy to testnet.

## What we look for in a pull request

- **Tests.** Every behaviour change comes with a test that fails without it. Tests live
  next to the code in `src/test/` and use `Setup` from `src/test/setup.rs`, which
  already deploys the contract, a token and a cast of accounts.
- **A stable interface.** Function names, argument order, event topics, event data and
  error codes are relied on by the backend indexer and the app. Don't rename or
  renumber them. New errors go at the end of the enum. If you really need to change
  the interface, say so in the PR and update `docs/contract-reference.md`.
- **No new panics on user input.** Return an `Error` instead. A panic is fine only for
  states that should be impossible.
- **Plain code.** Prefer a few more lines that read clearly over something clever.
  Comments should explain why, not restate what.

## Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org): `feat:`, `fix:`,
`test:`, `docs:`, `ci:`, `chore:`, with a scope when it helps, for example
`fix(subscription): ...`.

## Reporting security issues

Please don't open a public issue. See [SECURITY.md](SECURITY.md).

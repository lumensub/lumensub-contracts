# Running a keeper

A keeper is any account that calls `charge` on subscriptions that are due. In return it
receives the keeper reward, a share of each successful charge set by the admin (0.5% at
launch). LumenSub runs a keeper as part of its backend, but nothing stops anyone else
from running one, and more keepers make billing more reliable.

## What a keeper does

1. Find subscriptions that are due. The cheapest way is to index `sub_new`, `charged`,
   `chg_fail`, `paused`, `resumed` and `cancelled` events and keep each subscription's
   `next_charge_at` and status locally. `is_due(sub_id)` confirms a single one.
2. Call `charge(keeper, sub_id)` for each, signed by the keeper account.
3. Read the result. A positive amount means the charge went through. Zero means it
   failed (or was fully covered by credit) and the `chg_fail` event tells you why.

Past-due subscriptions stay chargeable until they lapse. Retrying them once every few
hours is plenty; most subscribers fix an allowance or top up within a day.

## What a keeper cannot do

- Charge before `next_charge_at`. The call fails with `NotDue`.
- Charge more than the plan price minus any credit.
- Charge a paused, cancelled, lapsed or completed subscription.
- Redirect funds. The merchant's share always goes to the payout address stored with
  the merchant, and the fee always goes to the treasury.

## Economics

A charge pulls up to three token transfers plus a storage write, so it costs more than
a plain payment. At 0.5% the reward on a 10 USDC subscription is 0.05 USDC, which
covers the transaction fee many times over at normal network load. For very cheap plans
the reward may not cover the fee; merchants pricing below about 1 USDC should expect to
rely on the LumenSub keeper.

## Races

Two keepers calling `charge` for the same subscription in the same ledger is harmless.
The first one succeeds and moves `next_charge_at` forward; the second fails with
`NotDue` and pays only its own transaction fee. Keepers that want to avoid wasted fees
can simulate the call first, which is free.

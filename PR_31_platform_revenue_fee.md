# feat(vault): implement platform revenue service fee (#31)

## Summary

Extracts a configurable percentage-based fee (in Basis Points, 10000 = 100%) from the expert's gross pay on every successfully finalized session, routing it to a designated treasury address. The user's refund is unaffected.

---

## Proof of Successful Build & Tests


![Test Results](<!-- ATTACH SCREENSHOT HERE -->)

---

## Changes

### `src/error.rs`

- Added `FeeTooHigh = 11` — returned when `set_fee` is called with a value above 2000 BPS (20%).

### `src/storage.rs`

- Added `DataKey::FeeBps` and `DataKey::Treasury` variants to the `DataKey` enum.
- Added `set_fee_bps` / `get_fee_bps` (instance storage, defaults to 0).
- Added `set_treasury` / `get_treasury` (instance storage).

### `src/events.rs`

- Updated `session_finalized` signature to include `fee_amount: i128` as a third payload field, giving off-chain indexers full visibility into fee extraction.

### `src/contract.rs`

- `set_fee(env, new_fee_bps)` — Admin-only. Rejects values above 2000 BPS.
- `set_treasury(env, treasury)` — Admin-only. Stores the treasury address.
- `finalize_session` — Updated payment calculation:
  ```
  fee_amount    = (gross_expert_pay × fee_bps) / 10_000
  expert_net    = gross_expert_pay − fee_amount
  refund        = total_deposit − gross_expert_pay   (unchanged)
  ```
  Transfers `fee_amount → treasury`, `expert_net → expert`, `refund → user`.

### `src/lib.rs`

- Exposed `pub fn set_fee(env: Env, new_fee_bps: u32) -> Result<(), VaultError>`.
- Exposed `pub fn set_treasury(env: Env, treasury: Address) -> Result<(), VaultError>`.

### `src/test.rs`

- `test_set_fee_and_treasury` — verifies admin can set both values.
- `test_fee_cap_at_2000_bps` — 2000 BPS accepted, 2001 rejected.
- `test_finalize_with_10_percent_fee` — full session: treasury=100, expert=900, user unchanged.
- `test_finalize_with_fee_and_partial_refund` — partial session: treasury=50, expert=450, user refunded 500.
- `test_finalize_zero_fee_behaves_as_before` — no regression when fee is not set.

---

## Acceptance Criteria

- [x] Contract correctly calculates fractional fees using BPS (`fee = gross × bps / 10000`).
- [x] Admin cannot raise fee above 20% (2000 BPS) — returns `FeeTooHigh`.
- [x] Fee is deducted from expert's gross pay, not the user's refund.
- [x] All 33 tests pass with zero failures.




## Closes

Closes #31

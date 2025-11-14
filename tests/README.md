# Tests

This workspace keeps integration tests under `tests/integration/`, grouped by purpose:

- `tests/integration/deterministic_build.rs` – ensures the SSG output is stable and deterministic.
- `tests/integration/test_conversion.rs` – validates tutorial content conversion and metadata wiring.

## Running the Suite

```bash
cargo test --tests            # run all integration tests
cargo test deterministic_build -- --nocapture
```

Longer-running or file-system heavy tests (like the deterministic build) live in the `integration` module so they can be split or skipped easily later. Feel free to add `tests/smoke.rs` or additional submodules if you introduce lighter checks.

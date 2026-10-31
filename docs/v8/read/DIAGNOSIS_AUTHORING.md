# Authoring the state diagnosis

Status: implementation guide for M-READ. The diagnosis framework is read-only
(ADR 0055): it evaluates the current simulation state without modifying it.
Checks, facts, subjects and their chains are data-driven from
`sim-core/data/colony/diagnosis.ron` (ADR 0033).

## Boundaries

| Concern | Source | Responsibility |
|---|---|---|
| Authored checks | `sim-core/data/colony/diagnosis.ron` | Predicates, severity, text, chaining, tags |
| Parser/types | `sim-core/src/colony/diagnosis.rs` | Schema validation, type resolution |
| Fact providers | `sim-systems/src/runtime/diagnosis.rs` | Read-only settlement/stockpile/hearth/room facts |
| Evaluation | `sim-systems/src/runtime/diagnosis.rs` | Predicate matching, chain following, caching |
| Bridge reads | `sim-bridge/src/ffi/diagnosis.rs` | `get_settlement_diagnosis`, `get_why_not`, `get_requests`, `get_current_task` |
| Locale | `localization/{ko,en}/ui.json` | `DIAG_*` and `REQ_*` keys |

## Add a check

1. Open `sim-core/data/colony/diagnosis.ron`. Find the subject block for the
   subject kind you want (person, stockpile, hearth, room, settlement).

2. Add a new check entry. Each check has:
   - `id`: unique string identifier (e.g. `"settlement_no_tools"`)
   - `when`: list of predicates, all of which must hold for the check to fire
   - `text_key`: locale key for the UI text (add to both `ko/ui.json` and `en/ui.json`)
   - `params`: map of display parameter names to fact names (values are resolved at evaluation)
   - `severity`: one of `"danger"`, `"warning"`, `"notice"`, `"ok"`
   - `next`: optional chain to another subject, with `kind` and `pick` strategy
   - `tags`: list of tags for matching requests to steward actions

3. Predicates use the mind's operator set:
   ```ron
   (fact: "hunger_tenths", op: ">=", value: Some(750))   // numeric comparison
   (fact: "cold", op: "has", value: None)                 // bool true
   (fact: "lit", op: "not", value: None)                  // bool false
   ```
   Available operators: `<`, `<=`, `>`, `>=`, `==`, `has`, `not`.

4. Add locale keys to both `localization/ko/ui.json` and `localization/en/ui.json`.
   Parameters in the text use `{param_name}` syntax.

5. Run `cargo test -p sim-core --lib "diagnosis::tests"` to verify the file parses.
   The `bundled_diagnosis_parses` test verifies all five subject kinds are present
   and every check has at least one predicate.

## Add a fact

Facts are named values read from the simulation state. Each subject kind has
its own fact reader in `sim-systems/src/runtime/diagnosis.rs`.

1. Add a `match` arm to the `settlement_fact` function (or the appropriate
   subject-kind reader when person/stockpile/hearth/room readers are added).

2. The reader must be read-only: it may read `World`, `SimResources` and
   `ColonyState`; it never writes, reserves or draws randomness.

3. Bool facts return 0 (false) or 1 (true). Numeric facts return i64 in
   documented units.

4. Return `None` for unknown fact names (the evaluator skips checks that
   reference unknown facts).

## Add a subject

1. Add a variant to `SubjectKind` in `sim-core/src/colony/diagnosis.rs` with
   `from_name`, `name`, `code` and `from_code` entries.

2. Add a subject block to `diagnosis.ron` with `kind: "your_subject"`.

3. Create a fact reader function for the new subject kind in the diagnosis
   evaluation module.

4. Wire the reader into the `build_chain` function so chains can follow
   `next` pointers to the new subject kind.

## Chain following

A check's `next` field specifies how to pick the next subject for the chain:
- `nearest_home`: for person -> stockpile, the stockpile nearest the person's home
- `home_hearth`: for person -> hearth, the person's settlement's hearth
- `parent`: for stockpile/hearth/room -> settlement, the parent settlement
- Other strategies can be added by extending `build_chain`.

Chains are followed to depth <= 5 (`MAX_CHAIN_DEPTH`). Only the first matching
check at each chain level is followed.

## Data-only test

The `data_only_check_addition` test in `sim-core/src/colony/diagnosis.rs`
demonstrates adding a check by editing only the data file:

1. Insert a new check entry into a test copy of `diagnosis.ron`
2. Parse the modified text
3. Verify the new check appears with its text key, severity and tags

This confirms that checks can be added by data alone without code changes.

# ADR0059 — Authored weaving and physical apparel

## Context

D-FEATURE-FIRST requires a working flax-to-worn-clothing chain before balance work. Existing food preservation cannot silently stand in for nonfood processing, and Stage1 format16 physical item layouts must remain decodable. Repeated held-out tuning is no longer the development process.

## Decision

Author recipes, reusable tool capabilities, passive process duration/capacity/water constraints, apparel protection/comfort/lifetime and knowledge markers in data. Keep ordered workshop-owned pending batches in a separate persisted ProcessingState. Consume inputs once, advance only qualifying ticks, retain blocked output ownership and book production only when items are placed. Carry exact remaining wear with each physical item through storage, carrying, equipping and death; only worn finite clothing ages.

Write format17 with explicit frozen format16 body and component conversion. Admit only the published predecessor catalog revision under preserved identities; legacy clothing gets unlimited wear, not rerolled new lifetimes. Append chronicle57/58 and clothing-comfort modifier16. Diagnosis/chapters/rendering read native facts; the single clothing card sends only a player's explicit dress selection and sends nothing on expiry.

Keep all initial tunables in commented data and BALANCE_DEBT. Prove the actual harvested→retted→spun→woven→tailored→worn chain with a scripted game scene, each-step captures and a completion assertion. Technical invariants, C3 and paired seeds0–7/48day collapse smoke block publication. No held-out or improvement tuning loops.

## Alternatives considered

- Reuse food fermentation state: rejected because nonfood batch ownership and migration must be explicit.
- Hardcode textile item IDs and recipe names in simulation: rejected by ADR0033; capabilities and metadata support authored alternate families.
- Reset wear on transfer/load: rejected because it duplicates useful lifetime and breaks physical ownership.
- Balance until improvement bars pass: rejected by the owner's feature-first decision.

## Consequences

Additional physical metadata requires exact historical readers and roundtrip tests. Pending outputs occupy finite capacity and survive save/load; dry or blocked processing stays visible in diagnosis. The scripted native chain physically completes harvest through both garments worn, with byte-identical save continuation. Authenticated format13 and format16 migration fixtures and focused save/apparel tests pass. Windowed captures, smoke and the full publication gate remain pending; these focused results do not claim publication acceptance.

## Date

2026-11-06

## Source

Owner D-FEATURE-FIRST; docs/v8/m6b/M6B_DECISIONS.md; docs/v8/BALANCE_DEBT.md; rust/crates/sim-core/src/colony/processing.rs; rust/crates/sim-systems/src/runtime/jobs/processing.rs; rust/crates/sim-systems/src/save/legacy_weaving_v16.rs.

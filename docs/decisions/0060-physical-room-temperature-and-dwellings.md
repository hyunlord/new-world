# ADR0060 — Physical room temperature and dwelling stages

## Context

Stage3 follows published weaving `90544f1f9`. D-FEATURE-FIRST requires working temperature and housing behaviour, not a natural-world comfort target. Baseline identity and genuine format17 clothing/processing remain compatibility boundaries.

## Decision

Use integer deci-degrees and one colony thermal sidecar. Reuse the authoritative room geometry and existing RoomSystem scheduling. Integrate each roofed room once per hour with positive authored capacity and loss, mathematical floor for negative values, outdoor temperature, actual occupants and one positively paid physical primary hearth. Record the existing fuel withdrawal; thermal integration never withdraws or burns fuel again. Unpaid, partial or zero-cost lit states supply no rated heat. Adjacent rooms initially lose heat to outdoors independently rather than exchanging it.

Author material insulation through a sim-side profile resolved against actual saved material identities, and apparel insulation against the item registry. Persist resolved rules without changing historical material, apparel or resource wire layouts. Use a colony format18 tail and an exact format17 reader; baseline retains its original format17 bytes. Pre17 physical item conversion is selected explicitly by version, never by “not latest”. New thermal state starts at the saved world's current outdoors, without invented past heat, fuel, exposure or upgrades.

S3-COMPAT-1 retains legacy colony Fire/Cold for a verified historical mind registry. Registration derives a nonserialized activation bit from the exact restored program; it does not rewrite saved policies or causal history. Current programs, including current custom policies, use numeric thermal. Loaded historical games await an explicit history-preserving policy upgrade rather than receiving new exposure without refuge behavior.

Actual successful passage opens a door for an authored duration; overlapping passages count their union. Path queries do not open doors. Place plus worn apparel insulation derives colony Cold. Hourly heat/exposure and per-tick felt refresh are distinct; hunger at priority130 consumes the previous committed Cold on the hourly boundary before RoomSystem152 commits the new temperature. Baseline keeps its old path and keyed draws.

Dwelling qualification uses real roof, beds, floor, paid warmth and reachable stored food. Earn upgrades immediately; save the beginning of continuous deficiency and downgrade only after the authored delay. Roof loss removes physical shelter immediately. Apply one stage mood effect per resident's actual dwelling, not one per bed. Append shared identities rather than reordering them; stage-specific mood interpretation remains M7 debt.

Floor coverage counts only tiles where ordinary floor construction is eligible: completed permanent building footprints and their work tiles are excluded from both numerator and denominator, even if a floor predates the building. The eligible area must be nonzero. This preserves the authored complete-coverage requirement without demanding construction that commands reject.

On load, a paid receipt for a surviving settlement must identify that settlement's physical hearth tile. An in-bounds tile belonging to another hearth is not sufficient. Stale receipts for removed settlements remain readable until runtime pruning; they cannot supply heat without a valid physical owner.

Project native values into the existing room/person inspectors, room geometry overlay, stage markers, diagnosis, chronicle and technology marker gauges. GDScript neither solves heat nor changes state when a panel opens. A real scripted construction/fuel/door/cooling/recovery scene and save continuation prove the feature; technical gates, C3 and paired seeds0–7/48day collapse smoke block publication. Held-out seeds1584–1615 remain unused.

## Alternatives considered

- Settlement-wide warmth or a free heater per house: rejected because one payment cannot heat unrelated rooms.
- A second room flood fill or a UI thermal solver: rejected because visible and simulated geometry must agree.
- Append fields to historic material/apparel structs: rejected in favour of a narrow persisted sidecar and genuine predecessor migration.
- Tune heaters or housing policy until natural comfort reaches95%: superseded by D-FEATURE-FIRST; initial values and limitations belong in BALANCE_DEBT.

## Consequences

The system has one physical primary-hearth heat owner per settlement and no inter-room heat exchange. S3-INTEGRATION-1 corrects the initial omission of autonomous heating service: existing stewardship constructs a real hearth refuge, authored need behavior seeks it, and an occupied cold refuge requests the existing paid fuel transaction during daytime. The refuge may reuse authoritative natural boundaries, which already have an authored heat-loss term. Ordinary bedroom construction keeps its stricter constructed-wall requirement. Refuge planning retains terrain, existing structures and stock zones, selects the actual primary-containing component with a legal built door and at least the authored minimum area, and sends ordinary construction commands. No additional heater or synthetic warmth is introduced. These connections are under implementation after the first collapse smoke failed. Implementation and verification are in progress; this decision is not a publication or test PASS claim.

## Date

2026-11-07

## Source

Owner D-FEATURE-FIRST; docs/v8/m6b/M6B_DECISIONS.md; docs/v8/BALANCE_DEBT.md; ADR0016; ADR0033; ADR0059; external stage3/predecessor17 and design/stage3-core-contract.md, design/stage3-runtime-contract.md.

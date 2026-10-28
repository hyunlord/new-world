# ADR 0052 — A conserved item food chain in colony worlds

Date: 2026-10-29. Status: accepted for M6a implementation; verification pending.
Authority: [M6A_GOAL](../v8/m6a/M6A_GOAL.md), ADR 0016 and ADR 0033.

## Context

V7 food is a renewing tile counter and an abstract settlement reserve. Colony
jobs already conserve physical items, but meat is converted back into that
counter. Seasonal harvest, cooking, storage and explanations require one
physical ownership model. Adding a parallel calorie counter would leave two
sources of truth and miss birth, porter, wildlife and automatic-meal writers.

## Decision

Keep the V7 baseline path, its supported save fixtures and registration intact.
In colonies replace the legacy food paths listed in M6A_DESIGN with food items.
Water, wood and sleep retain their existing mechanisms. Disable colony legacy
automatic feeding and meat-to-counter conversion. Colony fission is already
disabled; food porters cannot operate on an invented counter. The colony birth
withdrawal of ten abstract food units is retired; every child's actual hunger
is still a consumer and enters the planning demand.

Plant species are strict schema-1 data with stable registry IDs, resolver and
category fallback. A tile-indexed state table holds species, remaining yield
and next harvest date. Keyed placement consumes no stream state. Seasonal
phases and optional within-season ripe windows are data. Update plants only
at day/season boundaries, with cached spatial queries for agents. Harvest is
an ordinary reserved, cancellable job producing ground items; hauling uses
ordinary jobs. Never eat an infinite resource sentinel in a colony.

One food unit relieves thirty existing hunger points. Whole food items are
consumed; meals are not fractional. Cooked output counts in recipes produce
small one-unit portions, avoiding waste from a ninety-point item eaten at the
existing thirty/fifty-point thresholds. New nutrition groups, edibility,
preference, sickness and storage lifetimes are data. Cooked preference, then
missing recent groups, then distance determine the deterministic food choice.
The preparation category `meal` is not an extra nutritional group.

Cooking is an ordinary hearth bill requiring a lit hearth and authored
knowledge/capability gates. Grain parching OR a real pot enables porridge;
stew requires a real pot. The required recipes remain and a root cooking
recipe supplies a plant-based cooked path, justified by the pre-code supply
arithmetic. Drying uses an authored blueprint and elapsed-day transformation.
Recipes consume inputs and create outputs through the existing item ledger;
the food ledger records the satiety conversion difference explicitly.

Preserve the serialized layouts of ItemStack, Carry, Workshop and ColonyState.
A skipped `SimResources.food` sidecar owns plant state, stack birth days, actual
pot storage, bounded meal history and food tallies. Save format 13 writes this
sidecar explicitly after the existing body. Freeze the exact format-12 body
and admit only pinned known old bundled revisions, never arbitrary revisions.
Supported baseline saves load with empty food state and unchanged systems.
Old colony upgrades require explicit conservative inventory conversion and a
migration ledger, never silently replacing a stocked colony with empty stores.
Existing physical food gets conservative metadata from legacy meat lots in
sorted owner/oldest-lot order. Unmatched unknown-age pieces expire with an
explicit migration spoil ledger; they are never silently made fresh.

All food transfers move metadata atomically. Ground, carried, shelf and pot
ownership are distinct; a stable shelf key uses kind/material/quality bucket,
not a mutable vector index. Splitting copies the birth day; merging retains
the oldest day. Expiry is birth age against the current physical storage's
lifetime; moving never refreshes birth or resurrects an expired stack.
Spoilage destroys items with a recorded ledger term. Pot capacity counts actual
items in a real vessel removed from other ownership; filled pots are stationary
in M6a and spill preserved-age contents when removed. Vessel and contents both
remain conserved. A pot anywhere in the village does not confer storage on
unrelated stacks.

Winter acceptance need is `ceil(adults * 72 / 5)` food units; planning also
accounts for children and explicitly budgeted cold demand. Stores count real,
reachable edible item units. Harvest/cook/dry goals are demand-limited so
abundant plants do not become compulsory waste. Player overrides survive
steward planning. Pure mind facts expose availability, while registered
behaviors own target validation and successful consumption. Every same-tick
food mutation invalidates cached observations before the next person chooses.

The inspector, food reasons, warning and player-facing chronicle use these
same physical quantities. Plant phases use category-based drawn placeholders
until an art pass supplies bindings. No art generation is part of M6a.
The two requested variety mood variants are explicit M7 data-table debt.

## Consequences and verification

The sidecar minimizes old-wire changes but requires an exhaustive transfer
closure audit. Missing metadata is an invariant failure, not a fresh batch.
Full per-kind conservation and the satiety equation include harvest, cooking,
eating, spoilage, import/export and explicit old-save migration terms.

Plant updates consume calendar and registry/world state; they feed harvest
jobs and render snapshots. Food storage consumes items and rooms/containers;
it feeds meal choice and winter planning. Cooking consumes knowledge, hearth
state and inventory; it feeds nutrition and work/chronicle. Winter planning
consumes calendar, population and usable stock; it feeds job priorities and
alerts. These are the roadmap's multiple-input/multiple-output relationships.

The data-only apple fixture must use real placement, ripening, harvest, haul,
store and eating. Tests also cover interrupted ownership, expired inputs,
capacity, missing knowledge, day/season save boundaries and untouched baseline
continuation. Freeze bars before calibration; freeze policy/data before the
32 held-outs. The complete gate and clean-load performance are independent
requirements. No inherited UI-pass waiver certifies this simulation change.

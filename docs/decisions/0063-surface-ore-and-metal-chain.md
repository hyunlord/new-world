# 0063 — Surface ore outcrops and the metal chain

## Context

The approved C2 pass authors the Bronze column of CONTENT_BIBLE §4.3 in two
publications. Ore must have a physical origin; neither learning knowledge nor
opening a decision card may create metal. Existing workshop recipes already
accept multiple inputs, so fuel needs no separate production mechanism.

## Decision

Outcrop definitions use the species climate/terrain/elevation origin contract,
authored cluster sizes, material and output identities, and category fallbacks.
A separate keyed stream populates an ordered colony sidecar without changing
terrain, navigation, old resource nodes or baseline draws. Hidden deposits are
revealed by a nearby qualified practitioner or adjacent completed quarry work.
The discovery stores its real witness and projects only revealed rows to the UI.
Legacy saves acquire an empty ore sidecar; loading never rerolls their terrain.

C2-a supplies origins, discovery, rendering, Bronze data and existing production
paths. Mining, tin trade eligibility, steward requests and the complete causal
metal chain are C2-b. Chapter6 may read actual copper production in a; chapter7
must wait for a durable first-use witness in b. Data authored in a does not imply
that the b chain has passed verification.

The 82 historical knowledge nodes plus 47 new nodes exceed128 bits. A typed
256-bit mask preserves all u8 node ordinals. Personal practice masks are rebuilt
from authoritative entries when loading; neighbour upper words are explicitly
persisted in format22. The original lower128-bit wire fields remain compatible.
Compatibility requires the genuine published C1-b format21 save retained before
editing, alongside older supported fixtures.

Animal geometry and adult/juvenile silhouettes are authored data rendered by one
cached raster path. A pen marker and domestic label distinguish livestock from
its wild source without inventing a species. Wool uses existing recurring
livestock work and physical product placement. Handwritten render parameters
extend material looks without changing pinned imported art bytes.

Unsupported bee production, limestone supply and societal mechanisms retain
explicit Inert status and named debt. Existing ordinary recipe, construction,
trade, knowledge and job boundaries remain the implementation surfaces.

## Alternatives considered

- Dropping Bronze knowledge to fit128 bits: rejected because the full Bible
  column and data-only extension must fit without renumbering old content.
- Rerolling deposits into old saves: rejected because loading must preserve the
  saved world's identity and physical history.
- Adding a fuel scheduler: rejected because ordinary recipe inputs express
  ore, charcoal, moulds and handles already.
- New content enums or species branches: rejected because kinds remain data.
- Enabling lime recipes through an Inert gate alone: rejected because Inert
  nodes intentionally grant no execution gate. The authored recipe additionally
  requires its knowledge and unavailable physical limestone input.
- Branching the single existing upgrade edge: rejected as unnecessary scope;
  log houses upgrade to mud-brick houses, with stone houses still independently
  buildable. The historical upgrade path is retained in prior-save fixtures.

## Consequences

Frozen full prechecks precede every full gate. Baseline identity, deterministic
save/replay, exact accounting, no panics, tests/Clippy and C3 ceilings remain
blocking. Smoke compares both candidates with the preserved C1-b starting
binary on seeds0–7,48days, stewardON, no orders. Authored starting values are
recorded before results and are not tuned against smoke or origin seed results.

Real1920×1080 game captures and data-only family tests provide separate visible
feature evidence. Reports before the frozen gate are conditional; immutable
external receipts hold actual gate and remote publication results. This ADR
records the implementation decision, not a claim that either publish is done.

## Date

2026-11-10.

## Source

Owner-approved C2 GOAL.md and planner addendum, CONTENT_BIBLE §4.3,
ADR0033/0062, `docs/v8/c2/C2_DECISIONS.md` and `C2_REPORT.md`.

# ADR 0065 — C2-b metal chain: mining job, tin by trade, provenance by kind

## Date
2026-11-12

## Source
GOAL §3.4 (C2-b), plans C2-b-plan-ore / policy / provenance / save / ui, lead decision D-C2-B-1. Builds on ADR 0063 (surface ore) and 0064 (scripted chain preparation).

## Context
C2-a made outcrops, bronze knowledge, recipes and templates visible. Nothing could mine ore, bring tin into a no-tin world, or tell the chronicle where a first ingot's metal came from.

## Decision
- Mining is an ordinary designation and job on revealed outcrops: `Designation::Mine` (7), `JobKind::Mine` (26), `Command::DesignateMine`. All three are appended, so existing ordinals do not change. The output item comes from the outcrop's catalog `yield_item`. The job reuses the quarry rule's work time and batch size. Exhausted deposits keep their geology and discovery witness. The knowledge gate is `Designation("mine")` on `mining`.
- Tin goods come from data rows. `GoodsEntry` gains optional `requires_knowledge` and `absent_outcrop_tag` fields. Tin rows are eligible only when an adult practises `tin_sourcing` and no deposit, hidden or exhausted, yields a `metal_tin` item. `Command::TradeFromCard` is the ordinary barter, recorded with the card id that requested it.
- Provenance is recorded per item kind (`MetalState`: miners per yield item, ore/ingot imports, first-event witnesses), not per physical unit. Chronicle 67 FirstIngot, 68 ToolUpgraded and 69 FirstToolUsed cite their sources. The first ingot's cause is its first imported input's `Traded` entry. Milestones branch on item tags (`ingot`, `tool`, `metal_*`), never on content names.
- Tool upgrade: a held axe is swapped for a strictly faster stockpiled `tool`-tagged axe through the ordinary pick-up stop. Chapter 7 reads the durable first-bronze-tool-use witness.
- Save format 23 adds 27 completed-job counters and the metal tail. Genuine C2-a format22 saves load through an exact-revision adapter.
- Steward rules 12 and 13 extend `Bills` with an optional `stocked` gate and a `replace_tag` per-holder target. Mind facts `job_mining`, `job_smelting` and `job_casting` are pure observations, and older policy histories restore through `PRE_METAL`.

## Alternatives considered
- Reinterpret `JobKind::Quarry` for ore: smaller save change, but hides mining under quarry practice, skill and knowledge. Rejected.
- Per-unit metal lots across every carry, shelf, tool and trade boundary: exact batch ancestry, but touches about ten ownership seams plus validation. Deferred; the per-kind record is recorded as a shortcut.
- A guaranteed tin delivery or a new trade driver: rejected. Tin is an ordinary weighted staple row that a real caravan may or may not bring.

## Consequences
- A first-ingot line lists every recorded source of its inputs. It cannot name which miner's ore went into a given batch. Upgrade to per-unit lots if a line needs that precision.
- Charcoal is not hearth fuel (D-C2-B-1).

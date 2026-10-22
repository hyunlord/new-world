# ADR 0024 — Knowledge: data, component and gating

Status: proposed (M4 Part 0 draft, 2026-10-23; final in Part 6).

## Context
VISION §4: technology is owned by individuals and disappears with them (Henrich 2004; Boyd & Richerson). Until M3 a
person's competence was `Skills` (six kinds, level 0–20 from work ticks) and anyone could take any job. M4 adds what a
person *knows* beside how *good* they are.

## Decision
- Knowledge is data: `sim-core/data/colony/knowledge.ron` (schema 1), 24 nodes in file order (`KnowledgeId` = index):
  lane, era, threshold ‰, prerequisites, discovery difficulty and conditions, gates, marker weight, seeding.
- Each colony person may carry `Knowledge` (sparse entries sorted by id: proficiency ‰, last use tick, source, the part
  gained by observation) plus a cached `u32` practitioner bitset (bit i = proficiency ≥ threshold of i).
- Gating: a job whose recipe, blueprint or designation is covered by a node's gates is offered only to practitioners of
  every such node (`knowledge::gates()`, precomputed masks); orders are never refused and wait with reason
  `NoPractitioner`. Hauling, chopping, gathering and survival work stay open to everyone.
- Skill keeps setting speed and the quality roll; knowledge caps quality (`QUAL_CAP[proficiency / 100]`), and each
  completed gated job adds +2 ‰ and stamps the last use.

## Alternatives considered
- Knowledge as skill levels (one scale): cannot express "knows but rusty" or loss with a death.
- Dense 24-entry arrays per person: simpler, but 2.5× the memory of the sparse list at M4's typical 8–12 entries.

## Consequences
- A technology can be lost when its last holder dies or forgets; the colony must teach.
- Every new recipe, blueprint or designation is either gated by a node or listed as universal in the DECISIONS file.

## Date
2026-10-23

## Source
docs/v8/m4/M4_GOAL.md §3.1–3.2; docs/VISION.md §4; docs/v8/m4/M4_DECISIONS.md

# ADR 0024 — Knowledge: data, component and gating

Status: accepted (M4, 2026-10-23).

## Context
VISION §4: technology is owned by individuals and disappears with them (Henrich 2004; Boyd & Richerson). Until M3 a
person's competence was `Skills` (six kinds, level 0–20 from work ticks) and anyone could take any job. M4 adds what a
person *knows* beside how *good* they are.

## Decision
- Knowledge is data: `sim-core/data/colony/knowledge.ron` (schema 1), 24 nodes in file order (`KnowledgeId` = index):
  lane, era, threshold ‰, prerequisites, discovery difficulty and conditions, gates, marker weight, seeding, name key.
  Parsed once (`knowledge::defs()`); `knowledge::gates()` precomputes per recipe, blueprint and designation the mask of
  nodes that gate it.
- Each colony person carries `Knowledge`: a boxed slice of 6-byte entries sorted by id (last use day `u16`,
  proficiency ‰ `u16`, id in the low 5 bits and source in the high 3 bits of one byte, the part gained by watching) and
  a cached `u32` practitioner bitset (bit i = proficiency ≥ threshold of node i), refreshed by every write. 121–159 B
  a person measured (DECISIONS D10). Founders are seeded by the table at worldgen (`STREAM_KNOWLEDGE`; holders of a
  node also hold its prerequisites, D9). Saved (format v4).
- Gating: a job whose recipe, blueprint (its site, workshop or space piece) or designation is gated is offered only to
  people whose bitset covers the whole mask (`ctx.practice`); the person-independent memoised search falls back to an
  unmemoised search with the gate. Orders are never refused: bills, sites and marks without a practitioner wait, and
  `get_knowledge_waits` names the missing node. Hauling, chopping, gathering, foraging and survival work stay open to
  everyone (the universal list is DECISIONS D11). The V7 decision system's co-located construction arm does not see
  knowledge-gated colony sites (D22), so such a site is built only by a practitioner's `Build` job.
- Skill keeps setting speed and the quality roll; knowledge caps quality (`QUAL_CAP[proficiency / 100]`), and each
  completed gated job adds `use_gain` (2 ‰) and stamps the last use day.
- Everything runs only with `ColonyRule::knowledge` (`ColonyRule::M4`); the baseline world and the M1–M3 rules are
  unchanged (identity PASS).

## Alternatives considered
- Knowledge as skill levels (one scale): cannot express "knows but rusty" or loss with a death.
- Dense 24-entry arrays per person: simpler, but 2.5× the memory of the sparse list at M4's typical 8–12 entries.
- Refusing an order nobody can do: hides the problem; a waiting order with its reason shows the player what is lost.

## Consequences
- A technology can be lost when its last holder dies or forgets; the colony must teach (F10: 8/8 seeds lose the
  doorway 120 ticks after its only holder dies, and the door orders wait).
- Every new recipe, blueprint or designation is either gated by a node or listed as universal in DECISIONS.
- Any other path that advances work (the V7 arms) must ask the gate too; D22 is the one found in M4.

## Date
2026-10-23

## Source
docs/v8/m4/M4_GOAL.md §3.1–3.2; docs/VISION.md §4; docs/v8/m4/M4_DECISIONS.md D7–D11, D22

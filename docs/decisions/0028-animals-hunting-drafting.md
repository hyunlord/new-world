# ADR 0028 — Animals, hunting and drafting

Status: draft (M5 Part 0, 2026-10-23); final in M5 Part 7.

## Context
VISION §7: beasts are met RimWorld-style, by drafting people. The colony had no animals (`grep Animal` finds none at
M4), so the knowledge node `tracking` was lore only and hides had no source.

## Decision
- **Animals** (`ColonyRule::wildlife`, `WildlifeRule`): `Animal { id, kind: Wolf | Deer, hp, state, pos, pack }` in
  `ColonyState.wildlife` (not ECS entities, M5 DECISIONS D6). `AnimalSystem` (171, every 3 ticks): deer wander inside
  forest and plain tiles and (droppable 7.6) flee from people within 6 tiles; wolves walk a nav path to the camp, pick
  the nearest person outside a roofed room within 24 tiles, attack when adjacent, retreat when the pack has lost half,
  and leave after 2 days. Deer: 6–12 at worldgen in forest regions at least 20 tiles from the hearth, one more every 4
  days up to the cap.
- **Fights reuse the combat pair machinery**: `ColonyState.fights` holds `(attacker, defender)` pairs of `Fighter`
  (a person's `AgentId` or an animal id) with a progress count and the hit's damage; `CombatSystem` advances and
  resolves them after its V7 pairs (person deaths through `despawn_agent`, reason combat). Damage per resolved hit:
  wolf 6 (a person has 100 hp, a wolf 40, a deer 30), bare hands 4, a held stone axe 8, a ground axe 10, a spear 12.
  A person a wolf bites strikes back with what they hold.
- **Hunting**: the designation `Hunt` on a deer, or a hunt zone (a rect; any deer inside is marked), gated by
  `tracking`; `JobKind::Hunt { animal }` (`WorkType::Hunt`, the 8th skill `Hunting`): walk to within 2 tiles, attack
  after 60 ticks scaled by skill, hit chance 40 % at level 0 to 80 % at 20 (a spear doubles the chance class, capped
  95 %), deer hp 30. A kill drops Meat 3 (Meat 2 without a knife, ADR 0029) and Hide 1 at the deer's tile; chronicle
  `Hunted`. Meat on a stockpile cell in the hearth's reach goes into the settlement's food at 2 units each (eaten as
  foraged food is); meat spoils 6 days after the kill (ledger consumed).
- **Drafting**: commands `Draft`, `Undraft`, `DraftMove { tile }`, `DraftAttack { target }` (recorded and replayed).
  A drafted person's job is cancelled; `DraftSystem` walks them (`Seeking { Job }` with a `SeekTarget`) or holds
  them (`Consuming { Job }`), and the decision seam skips them (they are added to the job holders that may not yield).
  Needs still tick; at a critical hunger or thirst the person undrafts themselves (chronicle `Drafted`, cause auto).
  A drafted person without a weapon first fetches the nearest spear or axe within 12 tiles.
- **Spear**: recipe `haft_spear` at the wood bench (FlintBlade 1 + Log 1 → Spear 1, 60 ticks, woodwork, gated by
  hafting); a held spear lasts 30 attacks.

## Alternatives considered
- Animals as ECS entities: rejected (D6).
- A separate combat system for beasts: the V7 pair model (a pair, a progress count, a resolution) already fits.

## Consequences
- The decision system changes by at most +20 lines (the seam's inert set carries the drafted).
- A new item kind (Spear), job kind (Hunt), work type (Hunt), skill (Hunting); saves bump to format 5.

## Date
2026-10-23

## Source
docs/v8/m5/M5_GOAL.md §3 and §4; docs/v8/m5/M5_DECISIONS.md

# Temperament v1 (`game_v20`, Revitalize Pass 16)

The first piece of individuality in the game track: every person carries a
temperament of four axes, drawn at birth and partly inherited, and four of
their everyday choices lean with it. `game_v20` is a new scene; every older
scene (`game_v8` … `game_v19`, their `_nofission` / `_neutral` variants, the
recorder and the stress world) behaves bit for bit as before. The live game
(`WorldSimNode::init`, `reset_world`) runs `game_v20`.

## The model

Cloninger's temperament model (Cloninger, Svrakic & Przybeck 1993, *Arch Gen
Psychiatry* 50:975) describes four heritable dimensions:

| axis | name | what it is | the lean in `game_v20` |
|---|---|---|---|
| NS | novelty seeking | exploration, impulsiveness | how far a person wanders when idle; who goes with a splitting village |
| HA | harm avoidance | caution, worry, fatigability (HA4) | how early a person goes to bed |
| RD | reward dependence | sociability, attachment, need for approval | how fast a person grows lonely, hence how often it seeks company |
| P | persistence | perseverance despite fatigue | how full a load a person gathers before carrying it home |

Twin studies put their heritability around 0.3–0.6; this pass uses 0.5.

Each axis is an integer in permille (0–1,000, mean 500) stored in the ECS
component `sim_core::components::Temperament { ns, ha, rd, p }`. It is drawn
once, from the person's lineage key on the keyed stream `STREAM_TEMPERAMENT`
("TEMPRMNT") with replicate 0 — an identity trait, never redrawn by a noise
fork — and never changes afterwards. It is inserted exactly where `Hardiness`
is (founders in `worldgen/camp.rs`, newborns in `settlement_system.rs`), only
under `AgentRules::temperament` with keyed draws.

### Draws

- **Founder** `Temperament::founder(seed, key)`: for axis a (0 NS, 1 HA, 2 RD,
  3 P), `value_a = (u1 + u2 + u3) / 3` with
  `u_s = keyed_below(seed, STREAM_TEMPERAMENT, key, 0, 4a + s, 0, 1001)`,
  s ∈ {1, 2, 3}: mean 500, standard deviation ≈ 167 (the mean of three
  uniforms, a bell-shaped integer draw without forbidden math).
- **Newborn** `Temperament::child(seed, child_key, parents)`:
  `fresh = founder(seed, child_key)`; the mid-parent `mid_a` is the integer
  mean of the two parents' axes (one parent's axis when only one is known; no
  parent: the child is `fresh`); then
  `value_a = clamp(500 + (mid_a − 500) / 2 + (fresh_a − 500) × 935 / 1000, 0, 1000)`
  in `i32` arithmetic. Heritability 0.5; the factor 0.935 = √(1 − 0.5²/2)
  keeps the population's variance stable across generations when parents
  are independent draws.

### The leans (`TemperamentRule`, `rules.rs`)

With `d = axis − 500` (−500 … +500) and `/` the `i32` division (truncation
toward zero):

| axis | value it moves | formula | `game_v20` span | range |
|---|---|---|---|---|
| NS | an adult's idle tether radius (the fallback `idle_tether_radius`, every hour; not the evening gather radius, not children, not elders) | `max(2, base + span × d / 500)` | 4 tiles | base 8 → 4 … 12 tiles |
| HA | the start of the person's own night-rest window (end 06:00 unchanged) — every clock-filtered night-rest read for that person, the sleep branch included | `start − span × d / 500` minutes | 60 min | 21:00 → 20:00 (most cautious) … 22:00 (boldest) |
| RD | the loneliness growth rate in the `Social` component, set once right after the draw | `base × (1000 + span × d / 500) / 1000` | 500 ‰ | 0.04 → 0.02 … 0.06 per tick |
| P | the food and wood loads at which a worker hauls home (every production read of `haul_threshold`) | `clamp(base + span × d / 500, 1, 8)` | food 3, wood 2 (round 2; start 2 / 1) | food 4 → 1 … 7; wood 2 → 1 … 4 (clamped at 1) |
| NS | the fission departure rank | `(unit draw − weight × ns / 1,000,000, AgentId)` | weight 1,000 | NS 1,000 subtracts 1.0 |

The spans are section 6.2's start values except P's: tuning round 1 (seeds
16100–16115, 60 days) left T1-P at ρ 0.159 < 0.25 — with span 1 the integer
truncation moves the wood load only at P = 0 or 1,000, and a pickup takes a
bush's whole stock anyway — so round 2 raised food 2 → 3 and wood 1 → 2
(ρ 0.331 food, 0.346 wood on the tuning seeds). NS, HA, RD and the departure
weight were not tuned.

**Evaluation (seeds 16000–16031, 120 days, pre-registered):** NS (ρ 0.338),
HA (−0.578), RD (0.782) and the departure lean (+116 permille) pass; **P
fails**: food ρ 0.217 [0.147, 0.285] < 0.25 (wood ρ 0.395, reported without a
verdict). The food lean is clear among founders (ρ 0.334) and fades for
people born later in the run (0.250 before day 60, 0.139 after) — a
hypothesis, not a measured cause: later in the run the bushes near the
village hold less, so a load is set by what a bush holds rather than by the
threshold. The P lean is in the game but is weak for food. Note the truncation: an axis moves its value by one
step only when `span × |d| ≥ 500`, so a span of 4 tiles changes the tether in
steps of 125 permille, and the food span of 3 in steps of ≈ 167.

`game_v20_inert` is the placebo: the same rule with `effects: false` — every
function returns its base value, temperament is drawn and stored, and the
world equals `game_v19` (checked by fingerprint and digest at 40,320 ticks and
by 120-day probe summaries of 32 worlds).

## What is not modelled yet

- **Real parents.** Births still name as "parents" the two lowest-AgentId
  members of the settlement (`settlement_system.rs`); a newborn inherits from
  those two. Real couples are the family pass's job; until then T4's slope
  measures the arithmetic of inheritance, not a family structure.
- **The full pipeline of CLAUDE.md** (38-D polygenic core → TCI → HEXACO →
  traits → values → needs → emotions → behaviour → skills → roles). This pass
  is its first, reduced step: four TCI axes, no genes, no HEXACO.
- **Mutation by life events** (dramatic events shifting axes, latent vs
  expressed temperament): the component never changes after the draw.
- **Display.** Temperament is not shown in the game yet.

## For the art line

- `WorldSimNode.get_agent_temperament(entity_bits: int) -> Dictionary`
  returns `{found: bool, ns: int, ha: int, rd: int, p: int}` (permille
  0–1,000; `found: false` and zeros when the agent or the component is
  missing). `entity_bits` is the value from `get_agent_snapshot()`'s `ids`, as
  for `get_agent_detail` (which keeps exactly its nine keys).
- Suggested display in the inspector: four small bars (0–1,000, a tick at the
  mean 500), or only the dominant axis — the one farthest from 500 — as a word
  ("curious", "cautious", "sociable", "persistent", or their opposites when
  below the mean). All text through `Locale.ltr` with new keys in both
  `en/` and `ko/`; nothing in `scripts/` calls the getter yet.

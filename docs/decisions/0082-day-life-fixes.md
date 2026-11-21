# ADR 0082 — Day-life fixes read existing state and keep the save format

## Context

SIMAUDIT1 (`art-img-data/simaudit1/AUDIT.md`, v8/main 6ac56915b, seeds 0–7 × 60 days) found seven
close-up defects in an ordinary colony day: hut sleepers got `slept_outside`, jobs were taken just before
bed and dropped, talk partners were chased while busy, hunts were abandoned for needs, children flipped
between play and idle every tick, starving people stood idle and the steward re-issued a rejected hunt
zone. N2_PROPOSAL split the fixes over SIMFIX1, SIMFIX2, WORK1 and PURPOSE1; the SIMFIX1 brief takes the
seven quick fixes together and requires an unchanged save format.

## Decision

Every fix is a pure function of state the game already saves; no saved type changes.

- **Roof.** A sleeper is roofed in a roofed bedroom room or on a tile of a usable Dwelling whose template
  data says `function.warmth: Roofed`. Where a person slept is sampled only from the night-rest start.
- **Admission horizon.** The job layer admits a new job only when its estimate — Chebyshev walk legs × the
  move interval plus a work allowance, or `hunt.max_ticks` for a hunt — ends before the person's bedtime −
  slack and keeps fatigue below the sleep threshold − headroom. A job longer than `provision_over_ticks`
  waits until hunger and thirst last to its end; the `prepare_drink` / `prepare_eat` mind nodes top idle
  workers up at one meal's worth first.
- **Talk.** The invitation is the inviter's own saved `Seeking{Agent(invitee)}` state. A free invitee
  (idle or in a routine, no job, no need) gets the fact `social_invited` with the inviter as its peer, and
  the `accept_talk_invite_*` nodes walk it to the inviter. A chaser's partner target resolves only while the
  partner has accepted or is still free, and an untaken chase ends after `social.stall_ticks`, both with the
  existing retry cooldown. Free people seek a partner within a `social.peer_cell` tile block and pair off in id
  order; once a partner walks back the higher id waits for the lower. In the colony a talk lasts
  `social.talk_ticks` and pays `relief_total` out per tick. A chase chosen from social memory is no longer cut by
  `finish_social_need_*` the next tick.
- **Hunts.** A hunt ends with the existing `CancelReason::Stuck` after `hunt.max_ticks` or `no_gain_ticks`
  without closing in (Chebyshev distance to the prey; path replans no longer reset it), and the existing
  per-person skip list keeps the hunter off hunts for `retry_after_ticks`.
- **Flicker.** Routine places drop tiles that movement would refuse (impassable, or outside the region
  reachable from the settlement home) — facts and starts share that list. The visible-action projection
  shows an Activity as performed through its last tick.
- **Starving.** `starving_to_hearth` walks a person at hunger ≥ 90 with no reachable food to the band hearth
  (`rest:hearth`) instead of standing.
- **Steward.** The hunt-zone rule skips a zone that already exists, which `AddHuntZone` always rejects.

The tunables live in `data/colony/daylife.ron`, read once through `DaylifeRule::get()`. It is not a
`WorldRules` field, because rules are saved.

## Alternatives considered

N2_PROPOSAL's `MindState.invites`, routine cooldowns and a steward back-off table would each need a save
tail. A steward back-off for every rejected command would also need one; the only rejection that repeats
(SIMAUDIT1: 873 of 881) has a pure precondition. A GDScript two-tick pose hold has no consumer yet: no
renderer reads the visible-action rows (only `sim_reads.gd` and `renderer_agreement.gd`).

## Consequences

Colony behaviour changes on purpose, so colony identity digests change; baseline worlds are unchanged
(every reader gates on `rules.colony`). A loaded save runs with the build's `daylife.ron` values. The
starving node only walks people to the hearth; gifts between kin and eating the seed reserve
(`eat_seed_when_starving`, off) wait for HOME1 and the owner.

## Verification

`sim-test/tests/simfix1_daylife.rs` (data-only roof, admission horizon, hunt bound, chase and talk length,
node oscillation, steward rejects); the SIMAUDIT1 probe paired smoke, seeds 0–7 × 60 days, BASE against
candidate (`docs/v8/simfix1/SIMFIX1_DECISIONS.md`).

## Source

`art-img-data/simaudit1/AUDIT.md`, `N2_PROPOSAL.md`; `art-img-data/BRIEF_SIMFIX1.md`.

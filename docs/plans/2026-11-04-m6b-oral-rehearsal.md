# M6b oral rehearsal implementation plan

**Goal:** retain knowledge through actual evening conversation while teaching remains an explicit card action.
**Architecture:** extend the existing colony-only oral pair rehearsal; no new system, command, saved field, observer or proficiency gain. Preserve one rehearsed node per existing listener/teller pair, gathering eligibility, keyed pairing, ordinary learning and all decay thresholds.
**Tech stack:** existing Rust/hecs knowledge components and keyed RNG.

## Evidence and alternatives

Calibration02 seed1 loses hut_building, pot_firing, digging and grain_parching despite living last holders. Auxiliary grain practice has a separate concrete completion-credit repair. Existing oral rehearsal picks a uniformly random shared practitioner node. A pot-firing expert cannot rehearse with an oral learner at350 because the practitioner threshold is400; repeated telling also excludes learners already at oral_gain. Actual opportunities in this seed are not retained, so this design is a hypothesis to validate, not a demonstrated cure.

Restoring automatic Teach contradicts the adopted player-card scope. Raising proficiency floors or weakening loss bars would conceal missing upkeep. Instead let a real expert rehearse one already-heard concept with the paired learner: listener proficiency must reach min(oral_gain, node threshold). Rehearsal only updates last_used, never proficiency/source. Prefer the oldest last use in the pair, breaking equal ages with the existing keyed rehearsal draw. Other learning still reads the evening snapshot.

## Bounded execution

1. Add actual OralTransmissionSystem regressions in existing oral.rs: expert/learner rehearse without promotion; old knowledge precedes recently used knowledge; a lone expert receives no rehearsal. Run new tests against old production code and retain RED.
2. Replace only shared-rehearsal selection in oral.rs, update its header and D-S1 decision. Keep all other oral, decay and explicit teaching paths unchanged.
3. Run focused oral tests and existing systems suite, sim-test metric/knowledge tests, formatting and workspace checks. No fixtures or assertions weakened.
4. Build, strict baseline identity, commit/review push, then calibration03 on0–7 with prior failures preserved. This change must satisfy ordinary BASE-relative fixture safety and held-out bars before publication; no success is implied by unit tests.

Owner goal explicitly directs deciding/recording/continuing without another approval handoff. This design does not relax any acceptance threshold or spend held-out seeds.

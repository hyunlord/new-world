# M6a decisions

## D1 — Authorized scope and clock (2026-10-29)

Authority: the owner’s unattended M6a goal, committed verbatim as
[M6A_GOAL.md](M6A_GOAL.md). This replaces the completed M-UX2.1 task;
its historical step-41 waiver does not extend to M6a.

Start: 2026-10-29 07:38:23 KST. Target: 2026-10-30 03:38:23 KST.
Hard stop: 2026-10-30 13:38:23 KST (30 hours). Evidence and the clock
register are outside the repository at `/Users/rexxa/work/m6a-data/20261029`.
Questions are resolved conservatively or recorded in the final report,
not sent to the unattended owner. No scope has been dropped.

BASE is `5b0c88812374b87e7854cc9203f144ad0ccd9009`, obtained after the
required fetch and exact remote-report existence check. The registered
worktree is `/Users/rexxa/work/new-world-m6a-dev`, branch `v8/m6a`.
No other worktree is a build, edit or runtime surface for this pass.

## D2 — Documentation and design precede product code

`a3106c9d727ef10dd6f29cb2a5fe5d493e582565` records the immutable goal.
`cb3f79cc69df10ee05b10d5a5330e0226f67fc68` adopts roadmap v2, preserves
complete v1, and records the coordination rules. The documentation check
passed. The generated system-table check invokes Cargo; it was allowed to
finish before the separate census build. Builds and captures are serialized.

The Part 1 census is a zero-tick design instrument, not a calibration round.
The two independently executed runs cover seeds 0–7, eight 192×192 maps,
294,912 unique tiles and ten founders per world. Both exited 0 and had
complete terminal records; their bytes are identical (SHA-256
`6d52f6351fdc99d6d6d6bd4843ca092b1b4bb62a478f39ad276e28cefe77c322`).
Source, dependency locks and executable hashes matched before and after.
The census records ordinary final worldgen terrain, blockers and reachability;
it does not claim new plants exist in the product yet. The proposed placement
model will be checked against actual runtime placement after implementation.

## D3 — Acceptance and provenance

Calibration remains seeds 0–7, at most four logged 72-day cohort rounds.
No cohort round or held-out run has started. Held-out seeds 1296–1327 stay
unseen until policy/data and observer definitions are frozen. Failed and
partial runs remain evidence. Physical supply arithmetic does not establish
a probability of survival: the owner’s success-count bars remain acceptance
requirements, with the actual denominators and observation windows explicit.

Only the current full gate plus new food tests, mandatory baseline identity,
food acceptance and clean-load performance evidence can authorize publication.
No inherited M-CORE performance exception or M-UX2 stall waiver applies.
The UI-successor source-identity adapter cannot certify simulation changes;
a separate explicit simulation-successor validation contract is required,
with original adapters and failures preserved.

## D4 — Pre-code arithmetic revisions and migration review

The first placement model is preserved: seed5 has thirteen reachable roots,
three units per harvest and three spring harvests,117 units against144 nominal
warm adult demand. Before any cohort calibration, root yield becomes five
(195 spring units), stockpile lifetime ten days, and cooking produces whole
one-unit portions. Roots2->3 portions and fruit/grain cooking provide authored
plant-based cooked paths. Full harvesting wastes food, so design targets
usable demand and a real144-unit winter reserve rather than all available yield.

Independent checks verified32 storage scenario balances,2,304 targeted daily
rows, all seasonal production formulas, and the1235 harvested /35 spoiled root
model at stockpile lifetime10. Its44,440 work ticks exclude hauling and finite
execution; it is not runtime or diversity acceptance. The no-gain823-unit gross
bound does not pretend cooking's conversion gain is raw harvest.

The pre-code review checked all nine artifact hashes and required explicit
old physical-stack age migration. The design now preserves known oldest ages
and records conservative expiry of unmatched unknown-age food. No blanket
old-colony save rejection is allowed: existing format11/12 continuation tests
must pass. Actual implementation and full-gate proof remain pending.

# Stage C GitHub publication verification

At2026-10-25 20:46:16 KST root independently verified105 OPEN registry identities against GitHub, exactly one issue each, unique numbers513–617. Titles, complete body SHA256, category labels, open state and journal mapping agree. Existing issue59 retained number/title/body/state/labels/updated_at/comments/locked without change. No pre-existing issue was edited or closed. The registry anchor is committed7bce14c96c479e7e391c5a4716f92b8889a0b30d; its public availability follows C gate/publication and is not claimed here in advance.

Original publisher stopped twice after successful creation because the immediate listing lacked the new issue. Pending records prevented duplicate creation. Subsequent exact-marker reads found513/514. The corrected external publisher persists the canonical returned number and verifies direct GET; unknown outcomes use at most4 read-only attempts, never another create. Twelve mocked regression groups (including seven invalid/protected URL cases) passed, followed by actual105 completion and independent reconciliation. Same manifest/journal were retained across both failures and recovery. ISS-312 records this external-process incident; it is fixed and receives no duplicate GitHub issue.

External evidence under `/Users/rexxa/work/mux-data/stage-c-github-1/`: original execute.log/execute-resume-1.log, follow-up lists, execute-v2.log, v2-root-mock.log, journal/journal.json, final-all-issues.json, existing-59-before.json/after.json and final-verification.json. These local raw artifacts are not public attachments.

- Immutable manifest SHA256:31b9def5a72daa295a12d79bc7374a60fc02f30c7fc795b8609a665d8b86da3a.
- Final journal SHA256:1fd0078e5543b6e7466a31cbe8565dda96219ed62ed6b693776afd5b4bc82b45.
- Independent raw list SHA256:44a37b518767c78570b23fcb6ed86f84d8aa2a6bd04181d5aaef84e586f4ee97.
- Original publisher SHA256:fdab5ad3449389bda2c782ba586aa6f91cae370c7d4b3b642f6695e4b9504acd.
- Corrected publisher SHA256:b53f471d9da811cb178dd794773b10a3717546d3ee1e0734502d3b458fab15a7.

The original anchor has311 rows; the later fixed process incident adds312 without changing the105 OPEN set. Full C gate and branch publication are separate checks.

## Follow-up observation ISS313

C gate1's tail-latency observation was registered at clean `d113e2c9827c78cdf527432929bcac70e9c8b58a` as new OPEN ISS313, GitHub#618. Direct canonical-URL GET verified its exact body/title/perf label and open state; the immutable original105 issue bodies and journals were preserved. A subsequent independent all-state list verified all106 OPEN issues (#513–#618) and unchanged59. Raw evidence: `/Users/rexxa/work/mux-data/stage-c-github-2/journal.json`, `all-106-final.json`, `final-verification.json`. Final registry313=175fixed/106open/30deferred/2won't fix. The earlier105 observation remains historical, not silently rewritten.

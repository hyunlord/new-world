# Part 1 debug evidence

- Runtime: local Cargo debug test, no debugger/ports/watchers. World fixture seeds below 1000. HEAD 65ec44c9 with recorded Part 1 worktree changes.
- Failure: mutual activity peers produced one Started at 601 and one Completed at 604 with the correct causal parent; the subsequent assertion that interaction_progress was empty failed.
- H1: test expects cleanup one tick too soon. Distinguishing evidence: completed pair retains progress 3 at tick 604, next social tick removes it.
- H2: new targeted replay duplicates social starts. Distinguishing evidence: more than one Started/Completed event; the observed event-count assertions already refute this.
- H3: fixture contains a different active pair. Distinguishing evidence: map cardinality/key differs from the two fixture residents.
- Existing control: social_interaction_system.rs::completed_pair_is_counted_once_without_changing_social_outputs explicitly asserts retained progress at completion and removal on the next social tick.
- Planned change: correct only the new regression fixture assertions to check progress/key at completion and cleanup on the following social tick. No production change or debug prints.
- Artifacts: initial failure log retained as required by the brief; this journal under the allowed docs path replaces the skill default root journal. No temporary debug instrumentation or environment mutation is needed.
- Skill read: omo:debugging; Rust runtime; setup, investigate, fix, QA and cleanup references. Explicit unattended commit/publication authority in PASS17_GOAL.md supersedes the skill's generic confirmation handoff.
- Verification: existing social control 1/1 and invitation harness 4/4 passed. The same test runs old Idle and new Activity initial states: both retain exactly one progress value 3 at tick 604, both are Idle without Activity, and social tick 605 clears the map with social_completed still 1. H1 confirmed, H2/H3 refuted. No production fix, temporary instrumentation or debug process remains.

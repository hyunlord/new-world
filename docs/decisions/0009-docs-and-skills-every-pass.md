# ADR 0009 — Repository docs and skills are updated every pass

## Context
Direction, purpose and history lived in chat transcripts and 1.5 GB of pass evidence; lessons were scattered across reports and V7-era instruction files.

## Decision
README, VISION, ARCHITECTURE, ROADMAP, HISTORY, decisions (ADRs) and CHANGELOG live in the repository and are updated at the end of every pass, following the checklist in `.claude/skills/worldsim-docs-upkeep/SKILL.md` (also in AGENTS.md). A problem that needed more than one attempt, a better method, or unexpected Godot/gdext behaviour becomes a skill lesson in the same pass. Evidence images are not committed (at most 12 curated images in `docs/images/`, a folder created with its first image).

## Alternatives considered
Keep pass reports as the record (rejected: unreadable at scale, gigabytes of captures).

## Consequences
`tools/v8/check_docs.sh` (gate step) verifies required files, paths named in ARCHITECTURE, the generated system table, skill links and AGENTS.md rules.

## Date
2026-10-20

## Source
docs/v8/V8_DIRECTION.md §11.3–§11.4

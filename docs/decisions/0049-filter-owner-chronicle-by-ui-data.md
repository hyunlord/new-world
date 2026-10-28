# ADR 0049 — Filter owner chronicle by UI data

## Context

At seed0 after three elapsed days, the simulation records 38 mind-choice entries. They occupy 38 of the owner's last60 chronicle rows. The owner requests a quiet default list while preserving personal explanations and simulation records.

## Decision

Load kind visibility from `assets/ui/chronicle_feed.json`. Apply this generic presentation filter before retained-row capacity, so hidden records cannot evict visible history. Unspecified supported kinds remain visible. Keep notification delivery independent of list visibility. Personal story and Why views read the original records without this filter.

The capture also exposed a defence notification defect: defence entries do not advance the ordinary steward action sequence. Poll the chronicle cursor independently of that sequence, retaining rollback/load baseline handling and deduplication. This is a UI correction, not a new simulation event.

Protect composed Korean tool text with the existing word-boundary helper, including values inserted after localization. Keep fonts and panel widths unchanged.

## Alternatives considered

- Stop recording mind choices: rejected because simulation evidence and personal Why access must remain intact.
- Hardcode a mind-choice denylist in panel code: rejected by the data extensibility contract.
- Filter after retaining60 rows: would still evict owner-visible history.
- Increment the simulation action sequence for defence: outside the UI-only scope and unnecessary for notification delivery.

## Consequences

Data-only tests must demonstrate that changing the configuration changes retention. Runtime captures must preserve raw kind counts and personal explanations. Inspector reason text must fit at Korean UI scales1.0 and1.25; defence must show the real toast and drafted rings.

## Date

2026-10-28.

## Source

- `scripts/ui/panels/chronicle_panel.gd`, `assets/ui/chronicle_feed.json`.
- `scripts/ui/mux/mux_hud.gd`, `scripts/ui/panels/agent_inspector_panel.gd`.
- `tools/v8/test_core_mind_ui.gd`, `tools/v8/mux21_play_capture.gd`.

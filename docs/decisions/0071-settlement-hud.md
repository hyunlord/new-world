# ADR 0071 — Settlement HUD and evidence tooltips

## Context

The approved settlement layout reserves the centre for the world and keeps
orders, the chronicle and the inspector close to the corresponding edges.
The Korean and English interfaces use the same warm theme.

## Decision

Use one Godot theme with Pretendard for body text and Noto Serif KR for card
titles. Use existing commands and read-only bridge facts for the chips and
cards. Attach Why cards to the selected world target in screen space. Expose
additional tooltip levels only when the bridge supplies further evidence.

## Alternatives considered

An always-visible shortcut legend takes space away from the world.
Command buttons expose their shortcuts on hover and retain keyboard access.

## Consequences

Food depletion estimates depend on a daily harvest total. The person card
uses an initial badge until portrait art is available. The renderer and
simulation keep their existing data and behavior.

## Date

2026-11-18

## Source

Owner-approved UI1 brief and the three final layout mock-ups.

# ADR 0066 — Isolate the AI character trial from runtime assets

## Context

M3D-2a compares five AI-reconstructed subjects against M3D-1 placeholders.
Reconstruction, rigging and animation quality must be demonstrated before these
assets become game content. The shared DGX also runs the owner's research jobs.

## Decision

Keep concepts under `assets/3d/chars/concepts/`, trial tools under
`tools/3d/chars/`, and raw models/logs/captures in the external M3D-2a evidence
directory. Do not register trial assets in the runtime catalogue. Use one locked,
temperature-monitored DGX generation container at a time, with writable caches
and scratch inside its dedicated workspace. Preserve failed attempts.

Use M3D-1 humanoid bone names and one normalized rest skeleton per family, with
Blender automatic weights as the brief's permitted fallback. Bake original
texture UVs explicitly into the new atlas. Synthetic checks prove tool behavior;
real per-clip renders decide mesh acceptance. Gated model credentials require
the owner's authorized account; never replace conditioning weights silently.

## Alternatives considered

- Register trial assets immediately: rejected because input images and a
  synthetic tool check do not prove game-ready geometry or deformation.
- Run upstream setup scripts unmodified: rejected because their system package
  and temporary-directory writes exceed the DGX workspace boundary.
- Substitute unrelated image encoders for unavailable weights: rejected because
  the pretrained reconstruction model depends on its trained conditioning.

## Consequences

The game retains its existing assets until the comparison is accepted. The trial
can end with a documented installation/access failure without presenting fake
meshes or synthetic fixtures as generated characters. The full runtime gate is
distinct from this isolated asset experiment's checks.

## Date

2026-11-14.

## Source

Owner brief `/Users/rexxa/work/3d-data/BRIEF_M3D2_CHARS.md`;
external `/Users/rexxa/work/3d-data/m3d2-art/REPORT.md`;
`tools/3d/chars/dgx/README.md`; `tools/3d/chars/blender/README.md`.

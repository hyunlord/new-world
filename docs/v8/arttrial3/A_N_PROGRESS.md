# Route A north body evidence

All five north upright bodies are structurally accepted at 64x96. Each has
an RGBA PNG, grayscale region mask, and provenance sidecar in
`art/character/route_a/body_N_*`. No appearance/readability approval is implied.

| Frame | Generation attempts | Accepted attempt | Rejections |
|---|---:|---:|---|
| idle | 2 | 2 | Earlier foot-line rejection retained in raw ledger |
| walk_a | 1 | 1 | None |
| walk_b | 2 | 2 | try1: head_alignment |
| work_a | 3 | 3 | try1: head_alignment + hand_alignment; try2: head_alignment |
| work_b | 1 | 1 | None |

Nine total generations for five accepted bodies, 1.8 attempts/body. Seven
new frame generations were made in this lane; the two idle generations
preceded the handoff. No layer exceeded the three-attempt ceiling.

The built-in image generation edit tool received the accepted north idle raw,
adult_N guide, person_v2 and scene style references for every animation attempt.
Each sidecar retains the exact prompt, generation call elapsed seconds,
reference paths and SHA256, raw SHA256, final image SHA256, and mask SHA256.
Elapsed tool durations include tool scheduling/queue time; overlapping calls
must not be summed as exclusive artist working minutes.

Processing preserves the full source canvas, keys magenta, Lanczos downscales,
and removes alpha below16 using the existing `prepare_a.py`. For all four
animation frames, the authorized fixed-upper repair copies the idle head
rectangle `[20,32,44,50)` and torso rectangle `[18,50,46,70)` exactly.
No re-fitting, quantization, outline forcing, or whole-pose procedural redraw.

All visible skin receives mask1. A logged hand-coded polygon marks the plain
base loincloth as untinted worn item mask5. The polygon intersects visible
alpha and does not alter the artwork. Final layer checks and fixed-upper
checks pass for all five accepted layers. Rejected attempts and their
`check.json` remain under `/Users/rexxa/work/arttrial3-data/route_a/body_N_*`.

The idle provenance chain is explicit: N try1 used S idle raw plus guide and
style references; N try2 used N try1 raw plus those references. Originals were
preserved. Work motion is constrained and subtle; only the later moving-GIF
blind review can establish activity readability.

# ART-TRIAL3 decisions

## D1 — Scope and execution, 2026-10-22

The task-specific contract overrides the general v8/wip branch and end-of-pass
documentation rules: only `art/**` and `docs/v8/arttrial3/**`, on `v8/art-trial3`,
with no merge. Base HEAD verified as `2433a6128e89fde30a4dd9e6995fbfa75049ba30`.
Pre-existing `.gitignore`, `.ignore`, and `docs/art/` changes are not owned here.
The supplied goal is the approved execution plan. No further approval gate.

## D2 — Coordinate interpretation

Rectangles are half-open in game pixels; authoring is exactly 4x. Sleep is a
separate 96x64 canvas, with head left. Upright overlays are not applied to sleep.
Head/torso invariance excludes only the specified shoulder-line 2px margin.
The task's 64x96 canvas and 56px adult override the older VISION scale for this
isolated experiment. STYLE v2 is a style reference; Trial3 structure gates first.

## Execution ledger

| Part | Deliverable and evidence | State |
|---|---|---|
| 0 | Template, guides, contract, checker; 22 synthetic tests after independent audit | IMPLEMENTED; semantic limits recorded |
| 1 | 29/30 originals,58 derived;87/87 layers and603/603 combinations pass; furS missing | PARTIAL_AT_RETRY_CAP |
| 2 | Official CLI installed; authenticated workspace unavailable, no Meshy key | STOPPED_TOOL_UNAVAILABLE |
| 3 | 78pages and24GIFs; crowd78.8874FPS; timed playback unavailable | EVIDENCE_COMPLETE_R1_NOT_VERIFIED |
| 4 | S1 accepted87/87; S2 1.793; T1 2.542; numeric FPS78.8874; neither viable | REPORTED_PARTIAL |
| 5 | Korean goal §8 report, production guide, publication verification | IN_PROGRESS |

No structural failure may enter a normal board. No missing review or runtime
measurement may be reported as passing. Every attempted generation is retained.

## D3 — Independent structural audit

`STRUCTURE_REVIEW.md` found subset acceptance and sleep alignment gaps. The
checker now reports diagnostic subsets as NOT_COMPLETE even when their layers
pass. Sleep requires horizontal silhouette, bounding-box center(48,36) within
2px, and a declared left-side head box containing actual skin. This geometric
test cannot independently identify a semantic head; thin-head detection remains
a disclosed limitation of the specified bounding-box rule.

Sleep is one bare body render (the specified16 total bodies), not an upright
overlay combination. Its coverage_scope is bare_sleep_only. Visible sleeping
attributes must be answered as actually rendered, never inferred from unseen
assigned overlays. The lab and readability records must retain this limitation.

## D4 — Masks for the base loincloth

Body skin uses region1. The base loincloth is a worn item and uses region5,
which remains untinted. Cloth tint3/4 stays exclusive to top overlays. Initial
all-region1 body masks are processing intermediates, not final accepted masks.
Hand-coded mask corrections are explicitly permitted by goal §3.6 and must
preserve the original image and log exact polygon coordinates. This avoids
silently tinting the base loincloth as exposed skin.

## D5 — Route B stop

See ROUTE_B_PREFLIGHT.md: official CLI1.2.2 installed, but account status has no
workspace and workspace listing fails with no response and auth-login hint.
No Meshy key is available in inspected locations. B stops under the goal's
fallback rule, with zero submitted jobs. This does not assert the root cause
of the network/auth failure. Blender installation is deferred because there
is no usable 3D generation prerequisite. No B scores are fabricated.

## D6 — Allowed alpha cleanup

After whole-canvas Lanczos, top pixels with alpha248..254 become255, with RGB and zero-alpha silhouette unchanged. This is the explicitly allowed alpha cleanup, logged per asset; it fixes visible-body-through-top checks without masking their failures. No fourth generation or silhouette forcing.

## D7 — Diagnostic incomplete-set captures

S fur cloak exhausted three attempts. Do not relabel 87/87 accepted layers as a complete minimum set: adult,child,elder each miss this variant. Only the 603 structurally valid combinations may be captured using explicit --allow-incomplete, with NOT_COMPLETE and p1_eligible=false embedded. No full viable-route claim.

## D8 — Source metadata audit

Added missing mask_sha256 to five adult top sidecars. Their ten derived top sidecars retain the original derivation-sidecar hash and bind the new metadata hash separately; source image and mask bytes remain unchanged.

## D9 — Real-window capture corrections

The first oversized grid was clamped by macOS at1884px high; its partial images are retained externally and rejected as exhaustive grid evidence. Page captures retain exact1x/2x and validate actual viewport and global indices. All full pages stay external if the40MB repository budget would be exceeded; representative pages and complete index are published. A90-second crowd timeout is retained; monotonic wall time replaces accumulated frame delta for the60-second stop. These are harness corrections, not extra asset attempts.

## D10 — Explicit review limit and final verdict

Two fresh blind reviewers inspected all GIF decoded frames and durations, without route or key. They could not observe actual timed playback; therefore R1 stays NOT_VERIFIED, even where proxy accuracy exceeds a threshold. Final-source recapture reproduced all24GIFs byte-identically. No artifact or performance result cancels this missing gate or the capped missing cloak. Recommend commissioned2D base figures under the registered neither-viable rule.

## D11 — Visible rendering measurement

Godot on macOS skips rendering occluded windows while process ticks continue. The87-render/60s run is retained as invalid visibility evidence. A real always-on-top window, no force_draw, recorded4734draws/60.009588s=78.8874FPS,61/61drawable samples,0hidden frames. This is the standalone camp-prop crowd grid, not full camp.tscn or game simulation. Incomplete inventory keeps p1_eligible=false.

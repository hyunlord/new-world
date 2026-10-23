# POLISH2 portrait generation ledger

Status: **POOL_FROZEN_READY — all 204 selected portraits independently observed after the fixed repair round.**
Generation is complete at the 320-call cap. Pool/index writes stop for blind round 1.
Generation and visual acceptance remain separate; owner adoption is false.

- Built-in image generation only: **281 portrait calls, 281 completed outputs, 0 tool failures**.
  Actual markers record 204 first attempts and 77 second attempts. The maximum is two calls
  per variant; there were no third attempts. With 39 material calls, total use is
  **320/320**, with no remaining budget.
- The fixed 47-variant repair selection was executed exactly once per selected variant.
  The 47 repairs select try2 for 45 variants and preserve try1 for 2.
  All attempted images, including rejected repairs and their failures, remain immutable.
  The complete 204 selection contains 129 try1 and 75 try2 images.
- Canonicals are RGB 128×128, produced solely by full-frame RGB conversion,
  Lanczos resize and lossless PNG encoding. No cropping or pixel surgery was used.
  Largest selected PNG: 35,471 bytes.
- [GENERATION_MANIFEST.json](../../../art/portrait/polish2/GENERATION_MANIFEST.json)
  is the compact asset/provenance contract: 204 repository-relative portrait/native-tint
  paths, source-plan hashes, selected and latest attempts, actual prompt hashes,
  all attempted-call records, immutable processed revisions, observations and selection reasons.
  External provenance is explicitly marked. Manifest size: 1,291,755 bytes;
  SHA256 6053b121cf4d09965b99970ea0fab5989dce0035b0c68c68ea277dd7610bb3fa.

| Actual selected observations | Adult | Child | Total |
| --- | ---: | ---: | ---: |
| Geometry pass | 95/96 | 106/108 | 201/204 |
| Coarse hairstyle correspondence | 96/96 | 108/108 | 204/204 |
| Garment correspondence | 96/96 | 74/108 | 170/204 |
| Extra accessories observed | 0 | 0 | 0 |

No selected record remains PENDING_OBSERVATION. Geometry and source-correspondence
flags are distinct; numeric style measurements remain report-only.
Remaining selected geometry failures: adult_M_fur_cloak_bun/v1, child_M_fur_cloak_bun/v3, child_M_fur_cloak_bun/v5.
Unrepaired and exhausted failures retain their original thresholds and observations.
The exhausted adult_M_fur_cloak_bun/v1 and v6 still have observed full beards
despite their frozen clean-shaven briefs; hairstyle/top correspondence does not erase
those identity-axis failures.

The geometry-first rule preserves child_F_loincloth_bun/v4 try1: its repair improved
the garment but introduced a nominal 1px hair margin, versus the baseline 2px.
The rejected try2 geometry FAIL and consumed call remain visible in the manifest.
child_M_hide_wrap_bun/v1 also retains try1: the retry adds a sleeve-like shoulder patch
absent from the source and first attempt, leaving an ambiguous garment tradeoff.
Its actual try2 garment FAIL remains recorded.
The first edit pilot, child_F_hide_wrap_bun/v1, selects a visibly improved try2 while
retaining garment FAIL, as explicitly directed after actual source/try1/try2 comparison.
Every other repair has an independent, hash-bound comparison and explicit selection reason.

## Final immutable evidence

All paths below are external under
/Users/rexxa/work/artpolish2-data/portraits/generation/.

- call_inventory_r1.json — SHA256 a69b7fbff4bcfd32a00e9fa5e3835f66f04d9fcec0f8fe671f45ca4d3c6bc383.
  Actual 281 portrait call markers plus the 39-request material ledger; no queue-count estimate.
- final204_verification.json — SHA256 0f6906a73312168955672dac4fb2bf15604bfafecefe8f1dde1fa1e7dd180bf0.
  Verifies all 281 tool-original/raw/input hashes, all full-frame normalization pixels,
  selected request/reference/inspection/revision bindings, all 47 selection decisions,
  untouched unrepaired selections, preserved repair baselines and the 309-file reference freeze.
- Final candidates.json (one directory above generation) — SHA256 dff7f5b5dc87d22744f58466f60684217fa5106c5f01e195420c36ea5120284b.
  Collected once after all 47 decisions and all 204 selected observations were settled.
- Round 0 exact index bytes are preserved in round0_candidate_index_preserved.json,
  SHA256 b1e70036b014c032bfa4f9ddcafd142051475780d93e9faeb71bcfec2aac17af.
  Existing parent-owned blind snapshots remain untouched.
- Baseline audit baseline204_verification.json remains at
  SHA256 a9f857caa39ea2ac41f00f59aaf64504b830dc1fca48bcb02f3905a1aef40cf8.
  Its 196/204 geometry and 131/204 garment results are historical, not overwritten.
- Baseline call_inventory_r0.json remains at
  SHA256 dd2318dfad489453a103796b6295521573068711d88540865c923d17f410f8a3.

## Inputs and history

The stable reference freeze protects 309 files and remains
9cd4f807a1b42b1b021e143e85b357b1b3e058c6c2cf037b55b30b068d44d89a.
The frozen face plan remains 310a6de08234571c74df9845dc20d2f7c5377bb8aa2d0f861529d555f10f25b1.

Final-reference first attempts used the native tinted S/idle, style-only portrait anchor,
and exact 4× nearest enlargement. The final 60 unstarted child requests also used an exact
12× nearest upper-body crop with a source/hash-bound sidecar.
The fixed 47 repairs used all four references plus the preserved original full-resolution
try1 raw as reference 5. Source references control clothing/hair; the prior raw is edit context.
Each actual edit target was visually inspected before its call. The bulk 46 input manifests
also record that prerequisite; the earlier pilot manifest predates that metadata field.
Both previous-portrait and bust-detail provenance chains were verified before dispatch.

Every call preserves tryN.call.json (START before dispatch), untouched tryN.png,
tryN/raw.png, tryN/canonical.png, and exact request/reference bytes in tryN_inputs/.
Repairs first normalized externally without selecting; independent observations then entered
immutable processed records. Selection decisions retain both attempts and restore the exact
baseline canonical/selected bytes when the retry loses. Nothing is silently relabeled PASS.

The original 30 attempts were preserved through source-reference repair, followed by separate
try2 requests after the final freeze. Later unstarted male prompts clarified frozen beard modes;
unstarted prompts also clarified framing and source garment height. Four child clothing pilots
were ordinary first attempts; their literal failures remain preserved. The last 56 first attempts
used concise source-specific briefs, with stronger margin wording for the last 32.
Manual unstarted revisions archive prior request bytes and change records under prompt_revisions/.
Current generic prepare does not reproduce every prompt actually used:
**immutable per-attempt request snapshots and their actual prompt hashes are authoritative.**

One early child ready manifest was revised only in completion/overlay-check metadata.
Its exact original bytes and hash, final bytes and unchanged evidence entries remain preserved
in observation_manifests/ and observation_manifest_revision_005.json.
All later ready publication was atomic.

All originals and full-resolution evidence remain outside the repository.
No blind files were changed by this lane. Blind round 1 and owner adoption are not claimed.

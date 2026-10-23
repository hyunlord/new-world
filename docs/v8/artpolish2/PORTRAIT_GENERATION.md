# POLISH2 portrait generation ledger

Status: **POOL_FROZEN_READY — 204 / 204 portraits produced and independently observed.**
Baseline generation and pool/index writes are stopped for blind round 0.
Blind gates remain pending. Owner adoption is false.

- Built-in `image_gen` only; 234 portrait calls, 234 outputs, 0 tool failures
  at this checkpoint: 30 preserved original attempts plus 204 final-reference
  outputs. These include 30 separately requested second attempts. No third
  attempt or scorecard call was made by this lane.
- All 48 owner-sample variants are produced. This checkpoint contains 96
  adult and 108 child portraits. Generation and visual acceptance are separate.
- Actual call-marker inventory: 204 first attempts and 30 second attempts;
  maximum two calls per variant. With 39 material calls, the total is
  **273 / 320**, leaving 47 calls for a separately authorized repair union.
  Immutable external inventory: `/Users/rexxa/work/artpolish2-data/portraits/generation/call_inventory_r0.json`,
  SHA256 `dd2318dfad489453a103796b6295521573068711d88540865c923d17f410f8a3`.
- Canonical output: `art/portrait/polish2/<look>/v1.png` … `v6.png`, full-frame
  Lanczos normalization to 128×128; no cropping or pixel surgery.
- Final requests bind the calibrated native S/idle, style-only
  `art/style05/anchors/portrait.png`, and an exact 4× nearest enlargement.
  The 309-file reference freeze was verified at SHA256
  `9cd4f807a1b42b1b021e143e85b357b1b3e058c6c2cf037b55b30b068d44d89a`.
  Exact prompts, request paths, attempts and reference SHA256s are preserved
  in `/Users/rexxa/work/artpolish2-data/portraits/generation/`.
- The final 60 unstarted child requests gained a fourth reference: the
  source alpha bounds' upper three quarters, enlarged exactly 12× nearest
  with unchanged RGBA pixels. The sidecar binds the crop box and source/image
  hashes; every call verifies those pixels and snapshots the sidecar.
- Every request has `try1.call.json`, immutable `try1.png`, `try1/raw.png`,
  `try1/canonical.png`, and `try1/processed.json`. `try1_inputs/` additionally
  preserves the exact attached reference PNG bytes and request copy before
  any later mask repair. Original tool-output paths remain in call metadata.
- `checkpoint30.json` preserves the original checkpoint. The current
  `continuation_queue.json` binds all 204 final requests; completed rows have
  exclusive pre-dispatch `tryN.call.json`, raw originals, immutable input
  snapshots, processed evidence, and individual `tryN.ready.json` for later
  batches. `ready_checkpoint_204.json` records this checkpoint.
- Independent adult and child observations are processed with exact request
  paths and attempts, preserving earlier `processed.json` revisions. Actual
  margin/crop and identity failures remain failures. Numeric style results
  are report-only. All 204 selected portraits have observations; none remain
  `PENDING_OBSERVATION`.

| Independent observation | Adult | Child | Total |
| --- | ---: | ---: | ---: |
| Geometry pass | 94 / 96 | 102 / 108 | 196 / 204 |
| Coarse hairstyle correspondence | 96 / 96 | 108 / 108 | 204 / 204 |
| Garment correspondence | 89 / 96 | 42 / 108 | 131 / 204 |
| Extra accessories observed | 0 | 0 | 0 |

The complete candidate index was collected once after all observations were
processed: `/Users/rexxa/work/artpolish2-data/portraits/candidates.json`, SHA256
`b1e70036b014c032bfa4f9ddcafd142051475780d93e9faeb71bcfec2aac17af`.
The immutable audit is
`/Users/rexxa/work/artpolish2-data/portraits/generation/baseline204_verification.json`,
SHA256 `a9f857caa39ea2ac41f00f59aaf64504b830dc1fca48bcb02f3905a1aef40cf8`.
It verifies all 234 original tool-output/raw/input hashes, exact full-frame
normalization pixels, all selected request/reference/inspection bindings, and
the 309-file freeze. Canonical images are RGB 128×128; the largest is 35,471
bytes. All originals and full-resolution evidence remain outside the repository.

Runtime command (existing judge dependencies):

```sh
uv run --with pillow --with pydantic --with typer --with numpy --with scipy --with scikit-image python -m art.pipeline_polish2.portrait_process REQUEST RAW --attempt N
```

The original 30 were stopped for source-reference repair and preserved
unchanged. After the final freeze, the adult female short-hair pilot was
visually checked before continuation. Source-observed appearance briefs
replace garment/hairstyle guesses. A second, required adult male pilot
verified the clarified clean-shaven instruction before other unstarted male
requests resumed. Subsequent unstarted requests received clarified framing
and garment-height instructions; every started request still matches its
input snapshot. The actual geometry threshold is unchanged.

Four child clothing pilots are ordinary first attempts from the target pool.
The first three failed source garment-height correspondence; the fourth
repaired height but failed wrap topology by adding a raised round collar and
bound armhole. The last 56 requests used source-specific concise prompts;
the last 32 also received a stronger top-quarter blank-margin instruction.
Literal geometry and garment failures remain recorded. Manual unstarted
prompt revisions preserve exact previous request
bytes and revision metadata under each variant's `prompt_revisions/`.
The current generic prepare command does not recreate every prompt actually
used; the immutable per-attempt request snapshots are authoritative.

One observer ready-manifest was published provisionally, then changed only
its completion status and overlay-check prose. Its exact original SHA256 was
recovered and verified; both versions and the unchanged evidence entries are
preserved in `observation_manifests/` with
`observation_manifest_revision_005.json`. Later publication is atomic.
Blind gates and owner adoption remain pending.

# TRIAL3B Part 0 independent review — 2026-10-22

Verdict: **Part 0 checks reviewed; two acceptance loopholes found and fixed during review.** This is code/evidence review, not art or GIF acceptance. No route-A assets were edited by this reviewer. Scope: `anatomy.py`, `test_anatomy.py`, `check_layer.py`, `layer_schema.py`, `TEMPLATE.md`, and the route-A recheck artifacts.

## Findings and resolution

1. **Resolved: disconnected contact counted as contiguous contact.** The first implementation measured the bounding box of opaque overlap on a row, counting empty gaps. The current `anatomy.py` uses `longest_run` and therefore measures actual contiguous pixels. Independent counterexample: preserve body alpha support at 254 near the ground, leaving only two isolated opaque pixels at `(20,87)` and `(43,87)`. The current checker rejects `contact_ok`; those two pixels cannot establish a three-pixel contact run.
2. **Resolved: missing sleep anatomy silently accepted.** The first sleep fallback returned after the head bbox test. A solid rectangular slab on the sleep canvas, without limbs, passed `anatomy_findings`. The current implementation returns `limb_ok: sleep limbs NOT_VERIFIED: anatomical mask required`; the same counterexample fails through `check_layer`. Historical A sleep rows are now rejected, not silently counted as measured limbs.
3. **Minor documentation/code discrepancy:** TEMPLATE says all nine anatomical parts must be connected; the implementation checks connectivity for the eight extremity/limb labels, but only bbox area for the head. The owner's explicit head requirement is bbox area, so this does not invalidate the requested numerical check. Align the extra documentation promise or add head connectedness before presenting it as enforced.

## Evidence

- `python3 -m pytest art/pipeline3 -q`: **30 passed** at the review checkpoint.
- `git diff --check -- art/pipeline3 art/character/TEMPLATE.md`: clean.
- Independently recomputed all **265** SHA-256 entries from `/Users/rexxa/work/arttrial3-data/b/part0-amendment/a-before.json`: **0 mismatches**.
- Current `/Users/rexxa/work/arttrial3-data/b/part0-amendment/CHECKS.json`: **87 layers, 39 pass, 48 fail**. All three sleep rows fail. A was remeasured, not repaired.
- The hover, narrow hand/foot, thin limb and small-head tests alter pixel support rather than supplying asserted sizes. The new checks consume stored pixels and optional same-size anatomical label maps; sidecar widths cannot directly pass them.

## Limits to preserve

The historical upright A fallback measures fixed projected windows and cannot establish the anatomical identity of self-occluded parts. This is explicitly documented and must not become visual approval. B masks must come from the renderer's mesh/rig segmentation with provenance; hand-painted semantic labels alone cannot establish true anatomy. Head coverage is occupied **bbox area**, not filled-pixel area. Foot threshold is **length**, so the longest bbox dimension is consistent with §1.3. Contact is assessed only on upright bodies; sleep has no upright y=88 ground requirement.

Parts 1–4, renderer provenance, source licensing, actual animation playback and two-reviewer readability are outside this review and remain unverified here.

Leader follow-up: added connected-head regression and enforcement for anatomical
maps. Scoped anatomy+structure suite now31 passed. Earlier30-test checkpoint is
retained above as historical review evidence; the minor promise mismatch is resolved.

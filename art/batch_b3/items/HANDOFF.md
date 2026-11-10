# B3 item retries

One candidate passes the frozen A3 numeric style, projection and structural checks:
`art/style05/batch_b3/items/iron_spear.png` (64×64, icon anchor 32,32; footprint 1,1).
Try 2 is selected after the contact-sheet self-review rejected try 1 for weak small-size
head readability and numeric edge-density/saturation failures. The diagonal composition
retains a broad iron head at 32 px. Independent general-vision family review by icons_and_remainder passed (92/100);
see independent_review.json. The historical model judge remains MISSING; numeric success is
not misreported as a completed historical model judgment.

`steel_edge` remains unavailable. Its explicit steel-edge edit request was rejected
at input by the image service, request ID `f69e1e6b-1e51-4125-aad5-4ecb18ee36fc`.
No image was returned, and no follow-up rephrasing or workaround was attempted.

Counts: two generated spear originals, one selected candidate, one rejected original,
one blocked steel-edge request. Both A3 catalogue rows have empty count-bucket and
container-state lists; there are no required count or filled/empty variants.

Evidence: `/Users/rexxa/work/art-batch/B3/items/`. `final-check.json` recomputes all
raw, generation-record and normalized-image hashes and ENV1 normalization with zero
integrity/geometry failures. `records/` preserves exact prompts, references, tool hints,
seed=null and timestamps. `summary.json` includes source/reference hashes and rejects.
The two `boards/` files show all attempts against the six imported ENV1/POLISH items.

The import/bindings files are drafts only. After independent acceptance, pin the art
source commit and merge the single `assets` metadata row into the existing item adapter;
the importer does not consume adjacent sidecars automatically. Runtime numeric ID is
unverified and is deliberately null. No game assets or simulation files were changed.

Tool reuse: unchanged A3 `process`/`report` and ENV1 icon normalizer, with no new tool
code or dependency. Run external evidence with `PYTHONPATH` pointing only at A3 and
`cwd` outside either repository: mixed namespace imports from B3 cwd otherwise resolve
the STRUCT calibration in the wrong checkout. This initial processing error produced
no alternate art and was rerun with the correct import root.

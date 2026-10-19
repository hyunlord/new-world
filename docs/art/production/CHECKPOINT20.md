# Image checkpoint 20

20 calls, 320 cell attempts. Ledger records 10 accepted results including two preserved repeats; 8 unique production cells are installed (2.5% of attempts). This is below the 50% checkpoint threshold. Original per-call prompts remain in raw sidecars. Representative first-15 failures: 204 size, 182 fidelity, 128 missing idle, 32 invalid grid; reasons overlap.

Before: equal cells, generous margins and native pixel budgets, plus very-small guidance for small effects. Results repeatedly enlarge objects, add texture/new colors, or add rows.

After (one checkpoint wording revision, appended to all remaining prompts):

CHECKPOINT20 SIZE AND COLOR CONTRACT: Treat the sheet as a 256×256 logical-pixel board with exactly four rows and four columns, 64×64 logical pixels in each cell. Enlarge the WHOLE board uniformly with nearest-neighbor to the output square of1024×1024. A logical pixel must be a single uniform square, never add subpixel detail. Each figure/object stays within the stated NATIVE budget; adults max16×26 beforeoutline (100–130outputpixels height), children max12×20, small decals max10×10, other objects2logicalpixels belowtheir maximum. Preserve vast magenta margins; do not fillthe cells. Use8–12solidD3colors only, chosen from #0F0706 #2D1A12 #5F391D #6A472D #9F7F53 #C19256 #D69E58 #F1BF85 #D7BD77 #F6E6B0 #B80C07 #FB2A3A. Natural foliage may use #313625 #516A29 #90A034 #507C4E; stone #615D47 #AD9D7C. No extra saturated orange/yellowcream ramps. No internal texture, gradients, reflective highlights or shading specks. Actions: drink means kneeling with cupped hands to mouth; eat berries broughttomouth; sit groundkneesup; asleep lies horizontallyonfur; gatherreachesbush; buildstonehammerraised/striking. Precise listed4×4positions, neverextrafigures.

Acceptance thresholds and converter scoring remain unchanged. Very-small N3/F1/F2/U1 retries are exhausted; rejected cells may now use declared procedural masks. Literal converter JPEG/harmonic limitations are separately reported in converter_literal_audit.md.

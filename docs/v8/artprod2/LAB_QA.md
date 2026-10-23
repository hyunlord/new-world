# ART-PROD2 round0 lab QA

**Render fidelity PASS; semantic R1 not assessed.** Independent code/evidence audit; no reviewer response files were opened and no reviewers were contacted.

## Mechanical evidence

- All **576 actual Chromium screenshot crops** exactly match a decoded RGB state of their corresponding source GIF. Source size is 224×224 at 1× and 448×448 at 2×; device scale factor is 1. Every unique decoded state was observed.
- All **48 blind packet images** match the captured contact-sheet hashes. Their **384 image cells** exactly match the corresponding original screenshot crops, without decoded frames substituted into the capture evidence.
- All **192 authored PNG frames** independently reconstruct from accepted route-I sprites, S/E facings, meadow/forest-floor grounds and recorded frame plan. All **96 authored 2× frames** exactly equal nearest-neighbor enlargement of their 1× counterparts.
- All **240 source PNG/sidecar hashes** still match lab provenance. The 12 unique accepted looks include 5 female adults and 2 children, all tops/hairs and all five activities.
- The **96 preserved PROD1 originals and their prompts** retain all 192 recorded SHA256 values.
- No browser page errors were recorded. Capture targets 8 Hz; recorded timestamps remain the timing evidence, not a claim of uninterrupted exact-rate playback.

## Direct visual observations

Opened screenshot/contact-sheet evidence covering all 12 anonymous IDs, listed in `LAB_QA.json`. Figures stay inside their quadrants without visible clipping or opaque rectangular backgrounds. Front/side columns and green meadow/brown forest-floor rows are distinct. Children appear smaller; horizontal sleepers remain contained. The p1_12 observed sequence shows standing/stepping alternation, while p1_05 sleep remains static. Native 1× figures are small within the ground quadrants; 2× retains pixel edges.

The carried logs partly obscure hair in p1_04 and p1_09. This is a real semantic visibility limitation for the blind reviewers, not a transport/render failure. No activity, sex, age, apparel or hair accuracy score is inferred from this QA.

## Code audit and limits

Reviewed new `prod2_verify_new.py` and its test relative to `ebc24049`: no blocking finding. The verifier binds terminal intents to immutable inputs/initial outcomes, performs two independent re-evaluations, compares exact PNG bytes and path-canonicalized JSON, and checks that inputs and first evidence remain unchanged. Targeted test: **1 passed in 0.36s**.

Exact screenshot matching uses decoded GIF RGB states. GIF palette quantization is not required to reproduce lossless authored PNG RGB bytes. This audit certifies captured visual fidelity and cohort provenance only; it does not certify blind-review freshness, semantic R1, Godot P1, adoption, or continuous display scanout.

Reproducible external audit: `/Users/rexxa/work/artprod2-data/qa/audit_lab.py`. Full evidence and source hashes: [LAB_QA.json](LAB_QA.json). Raw measurements and images remain outside the repository.

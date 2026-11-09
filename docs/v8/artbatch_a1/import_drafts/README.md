# A1 import handoff — drafts only

Nothing in this directory has been applied to the game. Source inspection is
pinned to `origin/v8/main` **4a44336e7782f2b27615d942b5cd54dbfee89406**. Re-check
identities and integration behaviour against the C2/C3 destination commit before
using these drafts; C2 may add species after this snapshot.

| File | Intended use |
|---|---|
| `art_import.animals.json` | Replace only the full config's `animals` section, `source_branch` and `source_commit`; 20 sets / 216 selected frames |
| `bindings.animals.verified.json` | Proposed replacement for bundled `assets/v8art/bindings.json`'s `animals` section; verified codes 0–5 only |
| `bindings.pending.json` | Unmapped art and age/coat decisions; documentation schema, never paste into runtime bindings |
| `silhouette_handoff.proposed.json` | C2 §3.5 design values; explicitly not accepted `animals.ron` fields |
| `compatibility_check.json` | Read-only importer planning probe and draft checks; no real import or runtime verification |

`source_commit` is pinned to the verified asset commit `326822482b33edd1ac3bd4acefd0061f557756d6`.
The selected assets and aggregate animal manifest exist at that immutable commit. Do not use a moving branch name as the pin or claim an uncommitted
path can be imported. The draft is a fragment, **not a standalone importer
config**: retain all unrelated people/files/flora/meta/look-grid/budget sections.
The importer removes managed files absent from its resulting complete plan.

## Unified source and metadata

The source template is
`art/animal/batch_a1/adopted/{species}/{facing}/{pose}.png`, destination
`animals/{species}/{facing}/{pose}.png`, and metadata source
`docs/v8/artbatch_a1/animal_manifest.json`.

This includes the existing **20 wolf/deer frames**, copied byte-for-byte into the
unified source layout. Their poses remain unchanged: wolf has stand/walk/run/
attack/dead; deer has stand/walk/run/graze/dead. New sets add the catalogue's 184
core frames. The generated optional winter reindeer set is now selected with
another 12 frames, giving **196 new frames + 20 preserved frames = 216** across
20 sets. All 196 new frames passed independent visual review and source/structure checks;
all 216 delivered frames passed byte, anchor and coverage verification. They are
`ACCEPTED_ART_CANDIDATE`; owner approval and runtime import remain pending. Winter reindeer remains optional
relative to the bible's required family scope.

The importer reads `frames` from the metadata document and indexes by exact
`image` source path. Every selected source needs a row with `image`, `canvas_px`
and `anchor_px`. Upright is `[96,80]` / `[48,72]`; dead is `[96,64]` / `[48,56]`.
Keep hashes, footprints, provenance and acceptance results in the full art
manifest/sidecars even though this importer copies only canvas and anchor into
its generated runtime manifest. It does not enforce an adoption status field.

At the inspected commit, `tools/v8/art_import.py:182–190` loops the explicit
species, facings and per-species poses with no animal-name or pose enum. Thus the
full list works without changing importer code provided every source blob and
exact metadata key exists at the pin. It rejects a missing metadata key/blob and
duplicate destination. It does not infer or generate missing poses.

The initial compatibility probe exercised in-memory fixture bytes. The final
audit also ran the unchanged pinned importer `plan()` against actual blobs at the
asset SHA: all 216 animal images and metadata resolved, and all 66 unique pose IDs
referenced by verified binding codes existed. The full plan covered 899 source
files / 14,904,421 source bytes; this excludes generated importer files and masks
and is not a final on-disk pack-budget certification. No import writes occurred.
Evidence is recorded in `compatibility_check.json` and externally in
`/Users/rexxa/work/art-batch/A1/records/actual-import-plan-audit.json`.

## Verified bindings and limits

| Code | Registered identity | Registered serde name | Proposed art set | Current status |
|---:|---|---|---|---|
| 0 | wolf | Wolf | wolf | Existing binding preserved |
| 1 | deer | Deer | deer | Existing binding preserved |
| 2 | wild_goat | WildGoat | goat | Verified identity; new visual row proposed |
| 3 | wild_sheep | WildSheep | sheep_fleeced | Verified identity; new visual row proposed |
| 4 | wild_boar | WildBoar | wild_boar | Verified identity; **Inert** in source data |
| 5 | reindeer | Reindeer | reindeer | Verified identity; new visual row proposed |

Source: `rust/crates/sim-core/data/colony/animals.ron` at the inspected commit.
The rows preserve wolf/deer dictionaries and existing category/family fallbacks.
New grazer rows reuse the current deer LOD colour `[194,157,97]`; boar reuses the
current predator fallback colour `[184,67,54]`. Those are preserved fallback
colours, not sampled new sprite colours. No unverified code is assigned.

`AnimalState::code()` in `rust/crates/sim-core/src/colony/wildlife.rs` is
0 wander, 1 flee, 2 rest, 3 hunt, 4 attack, 5 retreat. New grazer mappings use
graze/run/stand/walk/attack/run; boar inherits the existing predator mapping.
The current `AnimalLayer.frame_id()` substitutes `still` for a moving pose when
the animal is not stepping. Importing attack art does not create attack behaviour
for a grazer, and an Inert boar binding does not enable the species.

These are **bundled binding rows**, not extension-pack aliases. The current
`ArtData.validate_extension()` accepts only `{code, alias_code}` animal additions
that alias existing bundled rows; it cannot introduce these new species visuals.

Important integration limits at this snapshot:

- `scripts/world/colony/livestock_layer.gd:81` still calls
  `Icons.item_texture(Content.row("animals", kinds[i]))`. Merely importing sprites
  and adding wild-animal bindings will not replace penned livestock placeholders.
- Animal binding selection uses kind, state and facing. It has no age, fleece,
  domestication or season selector. Kid/lamb/shorn variants cannot get separate
  invented codes; select them from the real kind plus explicit presentation state.
  Calf, pig/piglet and other future kinds first need verified authored identities.
- Dedicated young sprites already use 0.6 adult geometry. Do not multiply them by
  0.6 again. Reindeer young would need its own art or a disclosed adult-scale
  fallback; the current batch does not invent a reindeer-calf asset.
- Cattle and aurochs are separate art sets; only the destination content author can
  establish whether they use separate registered kinds or a domestic-state variant.
- Wild/penned markers belong to the renderer. Preserve anchors under E→W mirroring
  and the existing N→S fallback. Warm new textures before first visible use per C2.
- `art_import.py --verify` does **not** validate animal species/pose references:
  `binding_ids()` ignores those keys. Explicitly check every bound species/facing/
  state/still/dead pose resolves to an imported `animal_frame` ID.
- `tools/v8/test_art_import.py` currently expects `2 * 2 * 5` animal frames. Update
  that expectation in the importing pass to its actual accepted list: **216** for
  this accepted art-candidate list (184 core + 12 winter + 20
  preserved deer/wolf).

## Import-side completion sequence

With the A1 selection verified and committed, C2/C3 fetch the art branch,
merge the draft sections into their full config, set the exact asset SHA, and
resolve only verified bindings against their current content identities. Preserve
unrelated bindings and config. Then use the documented import flow:

1. Run `python3 tools/v8/art_import.py` against that full config, followed by
   `--check` and `--verify`; confirm the second import is unchanged and the whole
   pack remains within its 40 MiB budget.
2. Run the import/mask/extensibility tests with the updated actual frame count.
   Also enumerate every animal binding pose against the generated manifest because
   the existing generic verifier does not cover it.
3. Connect livestock and age/coat selectors only in the destination pass, warm new
   resources, rebuild the bridge before Godot, and run the destination's required
   import/play checks and native-scale captures of wild, penned, young and dead
   animals. These checks have not been performed by this art-only handoff.

No RON, renderer, live import config or `assets/v8art` file is edited here.

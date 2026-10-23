# ART-POLISH2 material review

Review date: 2026-10-24. Reviewer: independent read-only material/contract lane.
Reviewed base HEAD: `24d20e6b22aa42fb504f9bb5e56df7fca6024342`, plus the
uncommitted Part 1 implementation. References below identify the initial reviewed
code; fixes underway in parallel must receive a separate recheck.

Verdict: **CHANGES REQUESTED — four P2 defects**, no critical/P1 finding.
Generated texture repair, mud-brick data-only commit, final preview/owner strip,
and owner acceptance are pending independently. This review does not cover
portraits, sprite masks, or generation quality.

## Findings, ranked

### M1 — P2: different-material walls lose their shared topology

- Location: `art/compositor/material_layers.py:59-67`.
- `material_tiles` groups rooms by material key and passes only that group's
  cells to `wall_tiles`. A neighbouring wall with another material therefore
  disappears from the direction, exposed-face, end-piece, and ink calculations.
- Reproduction through the real scene renderer: create two single-cell rooms,
  `Room('north', frozenset({(0,0)}), 'log')` and
  `Room('south', frozenset({(0,1)}), 'stone')`, in an otherwise empty `Layout`.
  With the real `MaterialScene(read_catalog(root), SceneMaterials.read())`, the
  north tile reports `face_visible=True`, size `(64,60)`. Changing only the south
  material to `log` reports `face_visible=False`, size `(64,64)`.
- A wall directly south must remain a neighbour regardless of its material.
  Preserve all-layout occupancy as topology context while selecting textures
  for each material subset; test both vertical and horizontal mixed joins.
- Disposition: **PENDING**; root notified with exact reproduction.

### M2 — P2: narrow corners and crossings omit exposed ink boundaries

- Location: `art/compositor/walls.py:114-127`.
- Ink is drawn on the mask's bounding rectangle, with each whole side disabled
  when that direction has a neighbour. This loses the internal silhouette of a
  width-44 corner/T/cross. A cross with all four neighbours receives no ink at
  all even though its inset arms have exposed edges.
- Reproduction uses the existing adversarial `sources()` fixture, an opaque
  yellow junction, and `width_px=44, ink_edge_px=2, side_shadow=0`. For corner
  cells `{(0,0),(1,0),(0,1)}`, pixels `(53,30)` and `(60,12)` remain yellow
  `(255,255,0,255)` while adjacent `(54,30)` and `(60,13)` are transparent.
  The opposite exposed boundaries do get ink `(67,65,13,255)`. For the four-arm
  cross, all sampled inset edges remain yellow.
- Compute ink from the actual mask boundary, excluding only connected exits
  at the tile boundary. Keep a regression that verifies no ink seam at those
  connected exits. Existing narrow-top tests turn ink off and miss this case.
- Disposition: **FIX IN PROGRESS**, assigned to the material executor by root;
  not independently reverified yet.

### M3 — P2: an empty material override passes the schema and fails at rendering

- Locations: `art/materials/schema.py:51-58`,
  `art/materials/resolve.py:78-79`, `art/materials/render.py:31-35`.
- `TextureSet.safe_paths` rejects absolute/parent-traversing paths but accepts
  `""`. Base wall styles reject missing/empty required pieces, but partial
  material overrides use `TextureSet` directly, bypassing that requirement.
- Reproduction: take `read_catalog(root).model_dump()`, set oak's override to
  `{"log":{"face":""}}`, then call `MaterialCatalog.model_validate` and
  `resolve(catalog,'log','oak')`. Both succeed with `textures.face == ''`.
  `load_textures` then raises `IsADirectoryError` on the repository root.
- Reject empty/directory-like texture paths at the shared path boundary while
  preserving legitimate omitted override fields and inherited pieces. Add an
  invalid-override test through full catalog parsing rather than `model_copy`.
- Disposition: **FIX IN PROGRESS**, assigned to the material executor by root;
  not independently reverified yet.

### M4 — P2: emitted JSON Schema does not enforce the advertised parser contract

- Locations: `art/materials/construction_styles.schema.json:24-25` and `:165`,
  `art/materials/schema.py:74-93`, `art/materials/test_resolve.py:112-121`,
  `art/materials/ADDING_A_STYLE.md:79-88`.
- The exported schema references one all-optional `TextureSet` for every kind.
  Pydantic's model validators do not become JSON Schema conditions. The exported
  schema consequently accepts a wall with no textures, although the Python
  parser rejects it. Path and cross-document reference validators likewise are
  not represented by the current export.
- Reproduction executed with installed `jsonschema`: load the real style JSON,
  set the first wall's `textures={}`, `ends=null`, `corner=null`.
  `jsonschema.validate(payload, exported_schema)` succeeds;
  `StylesDocument.model_validate(payload)` raises `ValidationError`.
- The schema snapshot-equality test checks freshness, not equivalent behavior.
  Encode representable kind/path requirements and test malformed inputs against
  the exported schema, or explicitly document the additional mandatory semantic
  checks a Godot consumer must implement. Do not describe the export as a complete
  substitute for Python's validation. Cross-document checks may reasonably stay
  in a separately documented semantic validation stage.
- Disposition: **PENDING**; root and material executor notified.

## Nonblocking mapping note

`construction_styles.json` declares `packed_earth.sim_blueprints=["Floor"]`,
while the pinned simulation uses `BlueprintKind::PackedFloor` in
`rust/crates/sim-systems/src/runtime/jobs/space.rs`. Current output still reaches
packed earth through the kind default, but `style_source` reports a fallback
instead of the actual blueprint mapping. Correct the data key and cover it with
a resolver assertion before handoff.

## Verified evidence and boundaries

- Independently read material source records at
  `cb58a801d656291a04a5ebfb3be9805845ee6433`, the recorded `origin/v8/main` pin.
  All **105 source keys, 105 categories, and 105 raw djb2 u32 ids match** the art
  catalog; no missing or extra source keys. The hash was recomputed over UTF-8
  bytes using the pinned Rust contract, not inferred from list order.
- Resolver/loader/assembly contain no current named style/material literal in
  the existing AST audit. Resolver fallback and input non-mutation tests pass.
  Unknown categories use style tint; unknown material ids with known category
  use category tint/style. Partial nonempty overrides retain other pieces.
- Directional fixtures verify independent E-W/N-S sources and world-x/world-y
  phase, including negative cells. Corner/T/cross/isolated use the junction
  source. The 44px centre mask and shadow are tested; M2 is the untested ink case.
- South-face suppression for degree-three junctions is inherited byte-for-byte
  from STRUCT's predicate (`legacy_walls.py:111-112`), not a new regression.
  Existing face/end/base-anchor tests pass. This review does not reinterpret that
  historical rule as unconditional exposed-south-face rendering.
- Fresh command:

  ```text
  python3 -m pytest art/materials/test_resolve.py art/materials/test_material_tone.py art/materials/test_data_commit.py art/compositor/test_material_walls.py art/compositor/test_walls.py art/compositor/test_polish_walls.py -q
  61 passed, 1 skipped in 1.36s
  ```

- The skip is `test_published_style_commit_touches_no_python`: the required
  `DATA_ONLY_PROOF.json` did not exist. Temporary real Git histories prove that
  the checker rejects a `.py` file in the same commit and accepts data assets.
  This does **not** prove the actual mud-brick addition yet. Final delivery must
  recompute the proof on the real mud-brick commit and run without this skip.
- `prove_data_commit` proves the actual commit file list, absence of `.py`, style
  absence/presence across its parent, and committed texture-object presence. It
  does not validate pixel contents, seams, rendering, owner acceptance, or a
  particular requested style independently of its argument. Root must bind the
  actual final record to `mud_brick` and retain separate visual evidence.
- Texture images were being repaired during review. Missing/unrendered final
  assets, floor/door preview completion, new-era samples, final seam/projection
  gates, and owner strip are not claimed complete by these code results.

Only this review document was written by the review lane. No implementation,
portrait, mask, asset, or commit was changed.

## Final independent recheck — 2026-10-24

Current review verdict: **APPROVE GENERIC PART 1 CODE**. All four original P2
findings are independently verified fixed. The findings and their original
evidence above remain the historical first-round record; the dispositions below
supersede their pending status. No new blocking finding was identified in the
fixes. This is approval of the reviewed code slice, not a claim that the entire
art pass, generated assets, data-only proof, or owner acceptance is complete.

| Finding | Final disposition | Independently checked correction |
|---|---|---|
| M1 | CLOSED — verified | `material_layers.py:70-80` passes global occupancy and openings to each material batch. `walls.py:142-166` uses that occupancy for neighbours and the complete exposed-face set. The exact two-room probe now reports `face_visible=False`, `(64,64)` for the north tile with either log or stone immediately south. Horizontal subset tests also keep connected ends and side ink absent. |
| M2 | CLOSED — verified | `walls.py:114-129` derives ink from an eroded actual mask, extending connected boundary pixels before erosion. Original corner/cross probe points `(53,30)` and `(60,12)` now both equal `(67,65,13,255)`; connected exits `(63,8)` and `(32,63)` remain source yellow `(255,255,0,255)`. A full-width connected cross remains byte-identical to its source. |
| M3 | CLOSED — verified | `schema.py:15-16,70-84` rejects blank and noncanonical paths at catalog parsing. The original oak `face=""` override now raises `ValidationError` before image I/O. `render.py:35-38` also resolves filesystem paths and rejects a symlink escaping the repository root. The real symlink regression passes. |
| M4 | CLOSED — verified | `schema.py:17-33` exports per-kind required, non-null texture rules and wall end/corner requirements; texture paths have an exported lexical pattern. Independent `jsonschema.validate` against the actual on-disk `construction_styles.schema.json` rejects the original wall with `textures={}`. `ADDING_A_STYLE.md:79-89` now explicitly requires additional uniqueness, cross-reference, alias equality, file, and dimension checks in Godot. Schema export is no longer described as a complete substitute for those semantic checks. |

The nonblocking `PackedFloor` mapping note is also resolved: a fresh
`resolve(catalog, 'PackedFloor', 'clay', kind='floor')` returns
`style_source='blueprint'`.

Fresh independent test command, after inspecting the corrections:

```text
python3 -m pytest art/materials/test_review_regressions.py art/materials/test_resolve.py art/materials/test_material_tone.py art/materials/test_data_commit.py art/compositor/test_material_walls.py art/compositor/test_walls.py art/compositor/test_polish_walls.py -q
97 passed, 1 skipped in 1.04s
```

The new regression module contributes 36 passing cases. A separate focused run
of invalid override, symlink, external schema, connected-ink, and horizontal
topology cases reports **33 passed, 3 deselected in 0.21s**. Exact original
reproductions were also rerun separately from pytest, using the real catalog,
scene adapter, and emitted schema file. All assertions passed.

The remaining skip is still the actual published data-only commit proof. It is
expected before the subsequent mud-brick data commit, and must be eliminated
at the final pass gate. Texture repair and final rendered acceptance remain
outside this code approval.

Reviewed correction fingerprints (SHA-256):

| File | SHA-256 |
|---|---|
| `art/compositor/walls.py` | `3068674f151daa12b250385db908c78ea94755e42779a90ba2efc52753fea1fe` |
| `art/compositor/material_layers.py` | `17898d47f7c22a18b6a4a9f7ddc42838acaa2237badf2882c031a29f124a2e8c` |
| `art/materials/schema.py` | `b31c05b6292515e65e5ef0e50d94dece65005d97f6580346b4b03d6de4906078` |
| `art/materials/render.py` | `9004df0455562f49e8c3e41a700743012f3f539aa2b9cf01a78d11c6d8655223` |
| `art/materials/test_review_regressions.py` | `b1dbd7e31446748272427e84954d68fbb93fa3502d6557f2715318c99fbe39d2` |
| `art/materials/construction_styles.schema.json` | `5cb62f03449c7cb736c81f87a131ddf9667cc16ee64450c79b18df9307d06be3` |

Review lane edits remain limited to this document; no code or assets were changed
and no commit was made by the reviewer.

## Final board discovery: exposed T-junction face

The final six-material board exposed an additional assembly condition after the
initial independent review. A north/east/west T with no south neighbour lost its
front face because the renderer also required neighbour degree < 3. This
contradicted the goal's south-neighbour rule. A synthetic mixed-material subset
regression first failed with `face_visible=False`; the degree filter was removed
without changing global occupancy or the frozen legacy comparison.

Actual affected non-door cells in the frozen scene were (80,101), (81,101),
(82,101), (82,105), (83,105). Before images are retained in
`/Users/rexxa/work/artpolish2-data/boards/final_wall_topology_diagnostic/`.
The focused regression and the complete material/compositor suite are rerun;
raw red/green logs are `material-tee-red.txt` and `material-tee-green.txt`.
Ruff and basedpyright for the two changed files passed with zero diagnostics.
Final board inspection is recorded separately by the board lane.

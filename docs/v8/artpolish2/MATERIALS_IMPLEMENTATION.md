# Material implementation plan

Status: IN_PROGRESS. Scope: `art/materials/**`, generic compositor integration and tests only.

1. Freeze existing wall geometry with its existing tests and record their result.
2. Parse versioned style/material/category JSON once into frozen typed values; resolve purely with material → category → style tint fallback, kind-aware blueprint/default style selection, and optional per-style texture overrides.
3. Preserve the previous renderer as the explicitly selectable POLISH/STRUCT comparison implementation. Rewrite the public wall surface to render schema-resolved independent E–W/N–S/junction textures with world phase, exposed-edge ink, south-only faces, face-only ends, centred widths and side shadow.
4. Test unknown material/category/blueprint fallback, schema rejection, literal independence, both run phases, corners/T/cross/isolated, faces, and data-only commit proof. Drive actual textures through CLI test layouts when generation lands.
5. Document six-step style/material addition and handoff; freeze generic-code commit before adding later-era entries.

No new dependency, game edit, source pixel surgery, or source rotation. Generation belongs to the image lane. The old rendering comparison is intentionally kept byte-compatible for owner variant (a).

## Implementation evidence (2026-10-24)

Generic implementation is ready for the parent to commit before the data-only era
addition. This is not a claim that generated-texture gates or owner review passed.

- Baseline before rewriting: existing wall suites **31 passed**.
- Final scoped command `python3 -m pytest art/materials art/compositor -q`:
  **109 passed, 1 skipped**. The one intentional skip is the real published
  data-only commit proof, absent until the parent makes that isolated commit.
  It is mandatory to rerun without that skip at the final gate.
- `uvx basedpyright --pythonpath /opt/homebrew/bin/python3 art/materials
  art/compositor/walls.py art/compositor/legacy_walls.py
  art/compositor/test_material_walls.py`: **0 errors, 0 warnings, 0 notes**.
- `uvx ruff check` over the same scope: **All checks passed**.
- Programming no-excuse audit over that scope: **no violations in 12 files**.
- Tests first demonstrated the old renderer rejects material texture inputs;
  seven orientation/topology tests failed before the new dispatch/assembly.
  Pure resolver tests initially failed because that API did not exist.
- Adversarial tests distinguish E–W red-x and N–S green-y strips, including
  negative world coordinates, rather than inspecting metadata only. Separate
  tests cover corner/T/cross/isolated junction selection, absent narrow arms,
  face base y+43, face-only ends, 44px centring and soft side shadow.
- Resolver tests cover all six category defaults with unlisted materials,
  unknown categories, blueprint routing, kind defaults, partial texture override
  inheritance, raw simulation u32 lookup, and unchanged input JSON after repeats.
- Strict schema tests check emitted `*.schema.json` equality with the actual
  parser; a required AST-literal test checks the generic resolver/loader/assembly
  against every material/style id in the data (including later additions).
- The commit-proof checker is exercised against real temporary Git histories:
  seven texture objects plus JSON pass; one `.py` addition makes the commit fail.

Raw logs: `/Users/rexxa/work/artpolish2-data/material-tests.txt`,
`material-types.txt`, `material-lint.txt`.

## Manual surface exercise and current boundary

The real selected log/drystone source sets were loaded through `read_catalog` →
`resolve` → `load_textures` → `render_walls` for oak, pine, birch, granite,
limestone and sandstone. All six 5×4 open-door room PNGs were emitted at 320×252,
nonempty, with exact output hashes in
`/Users/rexxa/work/artpolish2-data/material-assembly-smoke/summary.json`.
The birch room and the composed owner-c scene were visually inspected. Directional
sampling, material tint, south faces and transparent aperture are present.
**These smoke inputs are diagnostic**: generation lane is retrying white face/end
gutters and initial E–W seam failures. Smoke rendering does not bless those art
assets. The packed-earth floor was still generating at this checkpoint, so the
complete `art.materials.preview` CLI (which also emits floor/doors) must run again
when its source exists.

The owner-c scene's thin pale lines on an unbuilt wall were traced to the existing
`drawing.site_overlay` construction-site alpha outline. A centred narrow top has
internal silhouette edges; the full-width version touches tile bounds. Those are
construction-site semantics, not missing material pixels or a reason to repaint.

## Files and handoff

- New `art/materials/schema.py`, `resolve.py`, `render.py`, `tone.py`, `preview.py`,
  `check_data_commit.py`; corresponding tests, three JSON inputs and JSON schemas,
  three prompt templates, `ADDING_A_STYLE.md`.
- Rewritten `art/compositor/walls.py`; previous comparison implementation moved to
  `legacy_walls.py`; new `test_material_walls.py`. Legacy output tests still pass.
- No `mud_brick`/`fired_brick` entry was added by this implementation lane. The
  parent must stage generated assets and JSON in its subsequent isolated commit.
- Actual material snapshot: 105 keys/categories and djb2 raw u32 ids at
  `origin/v8/main` `cb58a801d656291a04a5ebfb3be9805845ee6433`.
- `ADDING_A_STYLE.md` contains the exact six-step procedure, ID/category mapping,
  fallback order, JSON/Godot loading boundary, and unimplemented bridge-read need.

After the style-only commit:

```
python3 -m art.materials.check_data_commit COMMIT mud_brick > docs/v8/artpolish2/DATA_ONLY_PROOF.json
python3 -m pytest art/materials/test_data_commit.py -q
python3 -m art.materials.preview mud_brick clay /Users/rexxa/work/artpolish2-data/material-layouts/mud-brick
```

The actual proof file must be committed afterwards; its recorded SHA is verified
against Git content by `test_published_style_commit_touches_no_python`. No owner
acceptance, game integration, Rust identity gate, runtime game behavior or mainline
merge is claimed by these scoped art-compositor tests.

## Independent review repairs

The parent assigned four reproducible findings; each received an observed failing
regression before its correction. The initial new regression run was saved to
`/Users/rexxa/work/artpolish2-data/material-review-red.txt`; a separate symlink
escape fixture also demonstrated the old loader accepted an external source.

1. **Inset-top ink.** Bounding-box side ink missed narrow arm silhouettes, and a
   four-neighbour cross received none. Ink now comes from the inner silhouette
   boundary (mask minus its erosion). Padding extends only connected tile edges,
   so exposed inset edges are inked while connected arms have no seam. Exact
   reviewer pixels `(53,30)` and `(60,12)` now contain ink; neighbouring transparent
   pixels stay transparent. Full-width connected-cross output is byte-identical
   to its uninked source. The corner/cross output was rendered and viewed at 4×:
   `/Users/rexxa/work/artpolish2-data/inset-ink-review.png`.
2. **Texture paths.** The shared input boundary rejects blank, whitespace,
   directory-like, noncanonical, absolute, traversal, backslash, drive/URI and
   control-character paths. The actual file boundary checks resolved symlinks
   remain under the repository. Omitted override fields still inherit normally.
3. **External schemas.** Per-kind required non-null texture roles, required wall
   ends/corner, and lexical path constraints now appear in exported JSON Schema.
   The installed independent `jsonschema` validator exercises malformed inputs;
   snapshots alone are not treated as proof. The addition guide now enumerates
   the relational, uniqueness and file checks that Godot must implement in
   addition to JSON Schema validation. Schema is not claimed to replace those.
4. **Mixed-material connectivity.** The scene adapter now passes all wall cells
   and door openings to each material draw subset. Directions, face suppression,
   face ends and ink follow global occupancy. A real adjacent log/stone pair now
   gives the north cell a 64px top and no south face; a synthetic horizontal
   mixed-material subset also proves no internal end piece or side ink appears.

The pinned simulation blueprint mapping typo was corrected from `Floor` to
`PackedFloor`; its test checks `style_source == "blueprint"` so accidental default
fallback cannot hide it.

Post-repair validation: **145 passed, 1 skipped** (only the still-pending actual
style-commit proof); basedpyright **0 errors, 0 warnings**; Ruff clean; no-excuse
check **no violations in 14 files**. Evidence:
`material-review-green.txt`, `material-review-types.txt` under the external data
root. The edits include the now-assigned `art/compositor/material_layers.py`.
Texture generation sources were not modified by these repairs.

## Final generated-input CLI exercise

After the texture lane delivered the floor and final selected sources, the full
`python3 -m art.materials.preview STYLE MATERIAL OUTDIR` command exited **0** for
all six required pairs: log/oak, log/pine, log/birch, drystone/granite,
drystone/limestone, drystone/sandstone. Outputs are under
`/Users/rexxa/work/artpolish2-data/material-layouts/STYLE-MATERIAL/`: seven actual
wall topologies plus the resolved floor, both doors, and parsed resolver JSON per
pair. The final oak and sandstone rooms were viewed; former white face/end gutters
are absent. This completes the earlier floor-pending CLI exercise. Texture seam
acceptance remains separate: the texture lane reported exhausted final seam
failures for log E–W and mud-brick N–S. No transformation or threshold change was
introduced to hide those recorded failures.

## Dependency-free schema regression handoff

The final committed regression suite adds no `jsonschema` dependency. Its tests
independently assert the emitted per-kind required/non-null texture clauses,
wall end/corner requirements, version and extra-key constraints, and reject the
invalid path fixtures against every emitted texture-role pattern. These checks
supplement the parser tests and schema snapshot comparison.

The already-installed external validator was exercised separately against the
actual committed schema files: **1 valid document accepted, 18 malformed fixtures
rejected**. Reproduction and detailed results live outside the repository at
`/Users/rexxa/work/artpolish2-data/validate_exported_material_schema.py` and
`/Users/rexxa/work/artpolish2-data/material-schema-external-validation.json`.
The fixtures cover omitted/null required textures for all three kinds, invalid
paths, null wall aliases, unsupported version, and extra document fields.

Final aggregate command `python3 -m pytest art/materials art/compositor -q`:
**151 passed, 1 skipped**, recorded in `material-review-final-tests.txt` under the
external data root. The skip remains the pending actual data-only style-commit
proof. The review regression module contributes **37 passed**. Scoped
basedpyright remains **0 errors, 0 warnings, 0 notes**; Ruff and the 14-file
no-excuse check pass. No production implementation changed during this final
test-dependency cleanup.

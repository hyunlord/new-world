# Adding a flora species without changing Python

Production tree keys are exactly the simulation wood material ids from
`rust/crates/sim-core/data/material/wood/*.ron`. The JSON object key, `id` and
`log_material` must agree. A shrub has `habit: "shrub"` and may have no log material
or stump. Do not add a species-name branch to the resolver or compositor.

1. **Prompt and references.** Attach the route I person sheet and accepted
   structure/hut anchors. Use this template per size and shape:

   > One isolated {species}, {size}, shape {variant}, front three-quarter view
   > about 20 degrees down, upper-left light. Style 05 fine ink and restrained
   > wash. Healthy summer-green foliage using foliage_summer_* palette entries.
   > Visible trunk and roots, clear lower trunk below the crown. Transparent
   > background, no ground, no cast shadow, no label. Small height 64–96 px;
   > large height 128–192 px at 1×. Preserve the declared canvas and base anchor.

   Keep prompt, reference hashes, raw generation and all attempts externally in
   `/Users/rexxa/work/arttrees-data/`. Maximum three attempts per asset. Render
   ground shadows separately in the compositor.
2. **Generate summer and winter.** Produce both sizes and two distinct shape
   variants. For deciduous species attach each size's variant 1 as the bare
   winter edit reference and preserve its visible trunk silhouette. Evergreens
   retain their summer frame and receive code-generated winter snow. Generate
   the small species stump. Do not paint corrective pixels into a candidate.
3. **Masks.** Use palette Lab clustering to separate foliage from trunk/branch/
   ink. Supply a same-canvas mask per frame; bare frames and stumps use an empty
   mask. A mask is data, never a rule keyed on the species' name.
4. **Checks.** Run the unchanged `art/compositor/projection_check.py` with the
   declared anchor. Verify component count and no clipping. Summer canopy mean
   hue must fall within the foliage palette range, saturation ≥0.30. The lowest
   30% / central 40% trunk box must contain ≤2% mask pixels, and masks must cover
   ≥85% of foliage-cluster pixels. Keep failed attempts. A numeric pass does not
   establish visual acceptance. Run judge-v2 calibration before claiming its
   reports gate delivery.
5. **Data entry.** Copy a suitable entry in `species.json`, change object key,
   `id`, `log_material` and texture paths. Record actual canvas/anchor for every
   frame. `sizes.small` and `sizes.large` contain `variants` and `bare` arrays.
   `map_canopy` references `map_canopies`. Each season specifies `delta_lab`
   `[ΔL, Δa, Δb]`, `frame` (`foliage` or `bare`), and `snow_cap`
   `{depth_px, alpha}`. Snow alpha is at most 0.70. Spring `[6,0,6]`, summer
   `[0,0,0]`; choose documented autumn overrides. Deciduous winter selects
   `bare`; evergreen winter uses `[-8,0,0]` and upper-edge snow.
6. **Parse and resolve.** `read_catalog(repository_root)` validates the two data
   files. `resolve_flora(id, size, season, variant_hash, catalog=catalog)` is pure
   and returns `.path`, `.canopy_mask`, `.tint`, `.canvas`, `.anchor`, resolved
   material and canopy metadata. Negative hashes use Python's stable modulo.
   The resolver performs no file access and consumes no random stream.
7. **Render the strip.** Pass each resolved result to the season compositor for
   spring/summer/autumn/winter, beside a route I person. Inspect at 1× and 2×.
   Compare masked output with original: trunk, ink and alpha must be unchanged.
   Submit this strip with the schema validation and image-check evidence.

   ```bash
   python3 -m art.flora.boards --fixture path/to/species_entry.json --species-id species_id --output art/boards/species_strip.png
   ```

   The fixture is one Species record, not a catalog document. The generic
   command merges it into the parsed catalog for the strip only. Its sidecar
   records all flora Python hashes for the frozen-code comparison.
8. **Prove data-only addition.** Commit the generic code first. In a subsequent
   commit add only the species data fixture and rendered strip; inspect
   `git diff-tree --no-commit-id --name-only -r <proof_commit>` and assert no
   `.py` paths. `tests/test_alder.json` is the test-only example reusing existing
   frames with distinct spring/autumn shifts. It is deliberately absent from
   production `species.json`. Rendering success and a data-only commit are
   separate evidence, both required.

## Unknown ids and habits

A missing species id cannot encode its habit. Supply `habit=` from simulation
metadata. If omitted, `habit_defaults.json.unknown_habit` applies. The resolver
uses the habit's `fallback_species` textures and the **habit's seasons**, not
fallback species overrides. It reports `requested_id`, resolved `species_id`
and `fallback_used` so missing art never silently changes simulation identity.
`log_material` describes resolved art; simulation material ownership must retain
its original id independently. All three habit defaults are data records.

The JSON Schemas validate document shape. Typed parsing also enforces relationships
that portable JSON Schema cannot compare, including key = id = tree material,
map texture references, both sizes, four seasons and bare-frame requirements.

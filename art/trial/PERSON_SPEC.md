# Person composition contract

This is an offline trial contract, implemented by `art/pipeline/compose.py`.
Godot `agent_layer.gd` integration is deferred. Synthetic test images demonstrate
algorithm invariants only; they are not generated art or visual acceptance.

## Inputs and mapping

`Person` is a frozen typed value: `agent_id: AgentId`, `sex: male|female`, integer
`age`, `household_id: HouseholdId`, optional `hairstyle`, `skin` index 0–3,
`job`, optional `carry`, `state`, `direction`, optional `top`, `beard`, `head`.
IDs and age must be nonnegative. Default top is `hide_wrap`, default job `none`,
default state `idle`, default direction `south`. Optional layers default absent.

| Attribute | Exact mapping |
|---|---|
| age < 15 | `child`, regardless of sex |
| age ≥ 50 | `elder`, regardless of sex |
| other ages | male → `adult_m`; female → `adult_f` |
| household id | modulo 6 selects tunic/loincloth colour |
| hairstyle omitted | keyed agent-id choice among short, long_tied, braided, bun, shaved_sides |
| hair colour | independent keyed agent-id choice among four source colours; age ≥ 50 uses `#E8E8E8` |
| job | none → no tool; woodcutter → axe; builder → hammer; farmer → hoe; miner → pick; hunter → spear; gatherer → basket |
| carry | absent, logs, berries, stone |
| top | absent, hide_wrap, fur_cloak, woven_tunic, leather_vest |
| beard | absent, short, long |
| head | absent, headband, fur_hat, hood |

A supplied hairstyle overrides the keyed style for labelled variation experiments;
colour remains keyed. A shared child/elder body does not promise a visible sex cue.
Keep such uncertainty as a miss under the registered reading test, not a reason
to remove a question.

## Stable keyed choice

Hash ASCII `domain:agent_id` with BLAKE2b, key ASCII `worldsim-arttrial-v1`,
digest size 8 bytes; interpret digest as unsigned big endian and take modulo
choice count. Domains are `hair_style` and `hair_color`. This consumes no RNG
stream and never uses Python's process-randomized `hash()`.

Golden agent 7: style digest `4b5d4d47aa008cef` → index 0 (short); colour digest
`032fb793884f1566` → index 2 (`#8A6A3A`).

## Palette channels

Replace RGB exactly, only where alpha > 0; do not multiply or modulate the image.
Preserve alpha and all other material colours, including dark texture and outline.
Zero RGB underneath fully transparent pixels.

Channels are scoped by layer family: body uses skin/loincloth, top and headwear
use the top channel, hair/beard use the hair channel, and tool/carry materials
remain untinted. A gray stone is not a skin channel. Morphological closing can
copy unchanged body pixels into a layer; pixels exactly equal to the registered
body RGBA retain the body's skin/loincloth tint. This compares original inputs,
never repairs a generated variant or changes its extraction mask.

| Exact source channel | Destination |
|---|---|
| `#9A9A9A` | skin index: `#D8A884`, `#B9815F`, `#915C40`, `#68402E` |
| `#6E6E6E` | household colour, loincloth |
| `#C0C0C0` | household colour, top |
| `#B0B0B0` | hair colour (also beard) |

Household colours in order: `#EFE2C0`, `#6FB0E8`, `#E0604C`, `#50C0A8`,
`#B088E0`, `#E888B0`. Hair colours: `#2A1E16`, `#5A3A22`, `#8A6A3A`,
`#3A2A1E`. These are reserved entries in `art/palette/worldsim_b.json`;
the three additional skin tones are trial-authored, not source-extracted.

## Frame lookup and geometry

`resolve(Person) -> Appearance` resolves attributes without loading art.
`layer_keys(Appearance) -> tuple[str, ...]` exposes the exact ordered lookup keys.
`LayerAtlas(Mapping[str, PIL.Image.Image])` provides frames; `compose_person(person,
atlas) -> PIL.Image.Image` produces an RGBA sprite without changing atlas images.

Keys are `<family>/<variant>/<body>/<direction>/<frame>`, for example:

- `body/base/adult_m/south/idle`
- `shadow/base/adult_m/south/idle`
- `hair/long_tied/adult_f/east/walk_b`
- `tool/axe/elder/east/work`
- `carry/logs/child/south/sleep`

Families are shadow, body, top, hair, beard, head, tool, carry. Body and shadow
use variant `base`; other variants are listed above. No alternate names are
silently accepted. The atlas-building adapter must register extracted images
under these keys. No disk loader or renderer coupling is introduced here.

Mandatory upright frames: south and east × idle, walk_a, walk_b, work, all
32×32. Mandatory sleep: south only, 32×16. Four bodies × nine frames = 36 body
frames. Every selected layer must match that body's exact frame box and extraction
transform. All requested keys must exist: missing art raises `MissingLayerError`,
wrong boxes raise `CompositionError`. No resizing, pose copying, missing-tool
omission or synthetic fallback is allowed. Explicit `None` alone omits an optional
layer. Hair is always selected and required.

## Composition and directions

Draw shadow → body → top → hair → beard → head → tool → carry → outline pass.
The final inner 1px `#141210` outline applies to the composed foreground silhouette;
it does not outline the translucent shadow. Existing interior linework remains.
All layers share local coordinates; this compositor never crops or recentres them.

West looks up east frames, composes everything including the shadow, then flips
the complete output horizontally. Sleep always resolves to south without a flip,
regardless of requested direction. Sleep retains explicitly requested wearables,
tool and carry, so callers must set job `none` and carry `None` to put these away;
missing sleep layers are errors, not permission to reuse standing art.

Scene boards, 24-way uniqueness, palette membership of actual composites,
blind readability and generated frame alignment require real accepted assets and
are separate verification stages. Unit success is not evidence of those gates.

## Selected body registry and person shadows

The external `people/body_registry/registry.json` fragment contains 36 body keys
and 36 matching shadow keys. Selected generated sources are `adult_m_candidate_02`,
`adult_f_01`, `child_01_calibrated`, and `elder_03`. Body PNGs are byte-identical
copies of those selected frames. The calibrated child replaces the retained
22–23px candidate: south/east idle are exactly 24px; east walk B remains 23px as
natural pose variation. All other child upright poses are 24px. This is an
informed visual selection, not blind-readability evidence.

Person footprint shadows are deterministic derivatives, not generated bodies.
For each body's south/east four-pose union, use one shared ellipse: width
`max(6, round(0.6 * union_width))`, height 4px, centered 1px right of the union's
horizontal center and 1px above its bottom. This extends 1px below the feet.
Sleep uses the same rule on its separate 32×16 frame. Pixel-center ellipse
membership sets exact `#141210` at alpha 90; exterior RGBA is zero. The rule is
separate from the canopy-shadow 60%-area rule. No body pixels or per-pose
registration are modified. Shadow images match the body box and use the shared
canvas anchor `(16, box_height)`, not a measured foot position.

The fragment stores absolute external image paths. `provenance.json` records
source PNG/frame-manifest hashes, saved transforms and ellipse geometry; native
and enlarged contact sheets accompany it. Body references are the selected
source frames; shadow references are self references. Their zero colour-distance
scores demonstrate identity only, not independent style acceptance. All 72
entries pass palette/alpha/component/box checks; body entries additionally pass
outline checks. This fragment supplies no wearable, tool, carry or hair keys.

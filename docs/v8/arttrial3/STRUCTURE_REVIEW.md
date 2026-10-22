# Part 0 structural acceptance review

Read-only code audit, 2026-10-22. Scope: goal §2, template, contract, checker,
synthetic tests, and the first accepted route-A S-idle body. No production code
or pixels changed by this review. Findings describe the inspected snapshot;
later fixes require fresh verification.

## Evidence

`python3 -m pytest art/pipeline3/test_structure.py -q`: **17 passed in 0.52s**.
The five required fixtures exist. A disconnected floating arm, enclosed hollow
torso, shifted feet, and missing mask pixels are rejected. Declared hands must
also hit real visible skin: a declaration does not excuse missing pixels.

Additional temporary synthetic probes, using the existing fixture helpers and
the real checker, returned:

```text
VERTICAL_OFFCENTER_SLEEP Result(id='body', passed=True, findings=())
TWO_PIXEL_WIDE_HEAD Result(id='body', passed=True, findings=())
INCOMPLETE_ROUTE_PASS True
REPORT_COMBINATIONS 1
```

## Findings requiring fixes

### P1: an incomplete diagnostic inventory can become an accepted route

`art/pipeline3/check_set.py:79` permits `require_minimum: false`, then line 86
uses the remaining checks for the same top-level `passed` result. A manifest
named route A with directions S/E/N but containing only one adult S idle, one
top and one hair returned `passed: true` with one combination. There were no
E/N bodies, animation frames, sleep, second variants, props or derived bodies.
This is a real false-positive acceptance path, even though the default requires
inventory closure.

Fix: distinguish diagnostic subset success from route acceptance in the JSON
and Markdown; require inventory closure for route-level PASS. Keep subset
checking available without allowing it to qualify S1 or route viability. Add
the three-layer manifest as a negative route-acceptance test.

### P1: sleep bypasses template geometry and exhaustive composition

`art/pipeline3/check_layer.py:69` excludes sleep from all body alignment checks.
A vertical opaque strip at x=5..9, y=5..54 on the required 96x64 canvas passes
with an all-skin mask. It is neither horizontal nor centered at (48,36), as
goal §2.1 requires. `art/pipeline3/set_checks.py:75` then emits an unconditional
body-only passing combination for sleep and skips all top/hair/item products.
`TEMPLATE.md` says sleep is a complete body render, but this interpretation
does not prove the requested every-combination coverage or sleep attributes.

Fix: enforce pixel-backed horizontal extent and center/alignment for sleep;
define and check the actual sleep composition contract. Report sleep variants
that cannot retain the requested top/hair/carry attributes as missing instead
of counting a body-only synthetic success as exhaustive coverage. Add an
off-center vertical sleep rejection test and a sleep coverage/count test.

### P2: the accepted body's loincloth is classified as exposed skin

`art/pipeline3/prepare_a.py:35` labels every visible body pixel region 1 without
examining flat-color regions. The accepted
`art/character/route_a/body_S_idle.json` explicitly records this operation;
the image visibly has a loincloth. The contract says skin applies only to
exposed body and the loincloth remains untinted, but its current mask makes
the loincloth respond to skin tint. `check_layer.py` validates allowed IDs,
not the semantic correctness of that all-skin mask. This is not a mask-gap
failure: all opaque pixels are covered, with the wrong semantic label.

Fix: settle the base-loincloth region representation explicitly (the goal
reserves cloth IDs for tops), log the decision, and generate/hand-correct
actual exposed-skin masks. A body-wide constant region is not adequate
evidence of region-based tint correctness. Validate a tinted preview showing
skin changes without recoloring the loincloth.

## Additional acceptance limitations

- **Head shape is weakly constrained.** At `check_layer.py:75`, the head is
  merely all alpha above the head-box bottom. Replacing the whole head with a
  connected 2px-wide stalk at x=31..32, y=32..49 still passes. A shifted head is
  tested, but a missing-volume head is not. The literal bbox-within rule alone
  supplies no minimum size, so a stronger size/occupancy rule must be recorded
  as an explicit contract clarification, not silently invented or called a
  style failure. Do not claim the current gate proves anatomically complete
  heads.
- **Failure board localization is incomplete.** `check_set.py:43` documents
  diagnostic region marks, but the implementation paints one generic frame
  border and truncated rule text. It does not mark the failed hole, component,
  mask gap or coordinate region on the layer as goal §2.5 requests. Carry
  finding coordinates/regions into the board renderer and draw those marks.
- **Current anti-hole tests cover enclosed holes only.** That matches the
  explicit >12px² enclosed-region rule. A large torso omission connected to
  the outside is not an enclosed hole and is not proof of torso completeness.
  Do not broaden the reported guarantee beyond the implemented rule.

## Passing controls and remaining review boundary

Mask gaps, floating disconnected arms, enclosed torso holes, foot alignment,
shifted heads, fixed-upper-body differences, translucent top leakage, prop
clipping, top coverage and basic non-sleep Cartesian enumeration have direct
synthetic checks. These are useful controls, not proof of generated-art
anatomy, full route inventory, mask semantics, sleep attributes, or readability.
The accepted S-idle image was visually inspected; no taste verdict was made.

Recommended acceptance status for Part 0 at this snapshot: **changes required**.
This review adds only this document and makes no commits.

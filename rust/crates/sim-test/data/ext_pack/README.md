# Seven-family selected pack

This test-only append pack adds one item, recipe, blueprint with a distinct construction-style alias,
pressure, animal, knowledge node and alert/lesson. All art aliases reuse bundled binary assets.
Pressure/animal codes exceed signed 32-bit range; production consumers must retain their identity.

Select the descriptor before creating any world:

```
WORLDSIM_CONTENT_PACK=/absolute/path/to/ext_pack/pack.ron
```

The Rust integration target `data_only_ext` starts a fresh test subprocess for each scenario and uses
the real host loader. It exercises apprenticeship, gated crafting, construction/deconstruction/dig,
pressure effects and causes, generic animal spawn and ordinary hunting/declared loot, selected saves
and wrong-revision rejection. Negative children isolate duplicate, malformed, missing and overflow
sources in private temporary directories. They never mutate the parent's environment/catalogue.

After a fresh bridge build, the separate Godot driver `tools/v8/data_only_ext_test.gd` starts the actual
main scene under the selected pack and checks DTOs, resolved style art, real-world alert/button and
learning-helper behavior. Pass an absolute evidence directory outside the repository. Run windowed
to produce the screenshot; a headless functional result does not prove visual acceptance.

Raw run receipts and images belong outside the repository. The existence of this fixture is not a
PASS claim; the root coordinator schedules all builds/tests/Godot serially.

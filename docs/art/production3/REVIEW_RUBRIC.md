# Independent visual review

Inspect every shown slot. The sheet identifies its intended pose/view/object and provides raw, unfitted/fitted views, small-scale grass view, and (for characters) the body's reference idle and anatomical region map.

Return `ok` or `reject` per slot. A rejection has exactly one primary reason:

- wrong_pose
- wrong_view
- broken_anatomy
- different_person (hair, beard or build differs from the reference idle)
- wrong_object
- artefact (magenta remnant, text, stray texels)
- detail_destroyed_by_fit
- region_map_wrong (for example, face classified as clothing)

Style preferences are not rejection reasons. A technical score does not decide visual acceptance. For detail_destroyed_by_fit, separately state whether the raw/unfitted cell would be acceptable, so a scale decision can be audited.

For each slot write an object with sprite_id, candidate_id, verdict, reason (empty when ok), concise observation, and unfitted_acceptable (true/false when judging fitting damage, null otherwise). If no candidate is shown, identify it as unavailable rather than inventing a visual verdict. Directly open every assigned image before judging; use crops when required to see individual texels.

Art Pass3 addition: **The action must read at2× without the icon: carry shows the load, build shows the tool, drink shows hands at the mouth.** Review the displayed2× sprite itself, not just the large preview.

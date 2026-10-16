# Independent visual review

Inspect every shown slot. The sheet identifies its intended pose/view/object and provides raw, unfitted/fitted views, small-scale grass view, and (for characters) the body's reference idle and hair-mask overlay and white-hair preview.

Return `ok` or `reject` per slot. A rejection has exactly one primary reason:

- wrong_pose
- wrong_view
- broken_anatomy
- different_person (hair, beard or build differs from the reference idle)
- wrong_object
- artefact (magenta remnant, text, stray texels)
- detail_destroyed_by_fit
- hair_mask_wrong (adults only: incomplete visible scalp, long hair, reference beard, or non-hair recoloured; tolerate at most two uncovered hair-edge texels and two covered non-hair texels)

Style preferences are not rejection reasons. A technical score does not decide visual acceptance. For detail_destroyed_by_fit, separately state whether the raw/unfitted cell would be acceptable, so a scale decision can be audited.

For each slot write an object with sprite_id, candidate_id, verdict, reason (empty when ok), concise observation, and unfitted_acceptable (true/false when judging fitting damage, null otherwise). If no candidate is shown, identify it as unavailable rather than inventing a visual verdict. Directly open every assigned image before judging; use crops when required to see individual texels.

Action legibility: **The action must read at2× without the icon: carry shows the load, build shows the tool, drink shows hands at the mouth.** Review the displayed2× sprite itself, not just the large preview.

Colour-family rule: brightness differences that read as the same person under different light are acceptable. Reject different_person for skin/clothing changing colour family (tan to olive/green, cream to grey/blue). The white-hair preview must show the same person with white hair, no interior dark hair patches, and no white face/hands/feet/clothes. Children/asleep/lying frames intentionally have no hair mask.

For these boards, teal in the raw image is a mask key by design and is replaced in the natural previews. Judge different_person against the NATURAL reference idle at 2×, never against a teal reference. The preview labeled hair 6 is white; wood candidates use hair 4 (flaxen) instead. Both include black hair. Wood loads must read as sticks at 2× and be visible from behind.

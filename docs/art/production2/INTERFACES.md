# Art Pass 2 integration contracts

The GOAL is authoritative. This file coordinates implementation ownership, not acceptance.

## Asset provenance and runtime lookup

Existing body/view/activity/frame fields remain. Manifest and atlas carry source, qa_status, source_sprite (when derived), contains_basket, fallback_kind, activity_icon, drawable. Only independently reviewed, guard-passing cells receive qa_status=accepted. Derived/procedural entries remain explicitly non-accepted. Packer includes usable derived/fallback entries and excludes drawable=false. Runtime never picks another body's frame or a basket-bearing frame outside carry. Idle-icon fallback exposes its icon only on that lookup path and at zoom >=2. Installer, packer and renderer owners must agree before integration.

## Capture and nature

Tree/shrub/log sprites retain art_kind and tile_x/tile_y metadata. Capture records actual modulate alpha plus world anchor and draw order. A14 denominator is all forest tiles, with eligible-only coverage reported separately. A5 reads actual blended framebuffer pixels through alpha<=0.5 canopies; genuinely unmeasurable villagers remain failures. Static shadow sprites are not clearing objects. Clearings include settlement formation anchors.

## Reviewer isolation

Fresh art reviewer receives only review sheets, slot descriptions and the section7.3 rubric; no acceptance counts. Blind A11 reviewer receives only numbered crops and answer categories; no answer key, metadata or QA statistics. Labels are saved and hashed before answers are opened.

## Write ownership

- Leader: Part0 gate/lock/resource settings, inventory, task times, git/branches/push/PR, asset integration approval and final gate/report.
- Converter executor: v4 converter/tests; salvage candidates/review-sheet tooling. No installation until review verdicts exist.
- Agent renderer executor: frame lookup/variants/icon fallbacks and corresponding Godot tests (released in Part4).
- Nature executor: grove/clearing/canopy/season rendering and Godot tests (released in Part5).
- Capture executor: capture metadata, calibration, ingame QA/A13/A14 and report artifacts (released for Part7 preparation after baseline).

No agent commits, pushes or runs a shared full gate independently. Windowed runs are serialized through with_window_lock.sh. Existing Production1 documents remain untouched.

# Actual Part3 parent capture preparation — external only

Not installed, parsed or run. This supersedes earlier parent capture proposals for later execution without changing them. Requires installed Part3 inspector, its six new localization keys plus Part2's eight keys, registered activity getter and current debug bridge. No production edits are supplied here.

## Later one-launch command

Only after leader ends freeze and permits the capture, with sources/build/localization fixed:

```bash
GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
P10_DEADLINE_EPOCH=1792119780 \
python3 /Users/rexxa/work/art-production10-evidence/parent-capture-final-prep/run_parent.py \
 /Users/rexxa/work/art-production10-evidence/parent-part3-en-ko-NEW
```

Runner refuses an expired/different original deadline and an existing output directory. It observes quiet absence before launch and rechecks under the shared window lock. One owned process group bounds lock waits/quiet recheck/Godot to min(1800seconds, original remaining time), using the existing runner's termination protocol. It never acquires a quiet hold or removes foreign locks. Quiet absence is not a reservation against another task acquiring a later hold. No deadline extension or relaunch is automatic.

It records HEAD, working diff/status, source digest, exact external script and runner hashes, wrapper hashes, bridge dylib and source hashes, extension file, all five localization files. Before/after hashes and HEAD must match. These bindings detect drift; actual debug build-to-source correspondence still requires leader build evidence. A sourcefreeze agreement remains necessary because a before/after digest cannot detect a transient change restored during the capture.

## Fixture behavior and expected files

One main scene, one actual seed20261010 reset, frozen speed0/world processing disabled. Day30 10:00 derives from actual world_info (expected41880), not the old incorrect40560. ReadRoute advances exact ticks. Fixed1/60 is checked every process frame. After120 settling frames, the fixture records every live agent's actual family getter keyed by live AgentId/entity bits. It deterministically chooses the lowest visible parent with living partner and child.

If optional rings are absent, no relative-sprite/framing requirement is imposed: family links remain real, and sidecar status is `dropped_absent`. If present, require visible partner and child and fitting Z4 bounds; record visible rings and require they refer only to live partner/children with alpha below1. Installed optional rings still need lifecycle/visual acceptance; zero visible rings is not inferred PASS.

Use actual Locale.set_language for en and ko in the same launch and restore the original language afterward. For each language, emit the actual partner button's pressed signal and first living child's pressed signal; after processing, verify selected entity bits and get_agent_detail.AgentId against real live snapshot and actual family membership. Restore parent selection before screenshots. Four click records verify production signal routing and selection; **mouse pointer hit-testing is not tested**.

For each language save three1920x1080 screenshots:

- `parent-en-top.png`, `parent-ko-top.png`: inspector top, original needs/details.
- `parent-en-temperament.png`, `parent-ko-temperament.png`: scroll to temperament to avoid relying on it fitting the top view.
- `parent-en-family.png`, `parent-ko-family.png`: scroll bottom for family; existing620px panel stays fixed.

`parent.json` records exact language, frame/tick, scroll offsets, label/button texts and rectangles, selected family/census, temperament getter, camera, optional-ring state, four live click results and fixed-delta sample count. Full label tree visibility does not account for ScrollContainer clipping; screenshot review remains required. Runner validates six view/locale records and PNG sizes, live parent membership, four click identities, tick41880, panel height, delta, optional-ring membership, terminal errors and source stability. `capture_contract_pass` never grants visual acceptance.

## Visual QA checklist after terminal success

Review the six original PNGs at native size, with parent.json available for identity checks. Record per-image PASS/FAIL with concrete coordinates/crops, without rewriting raw captures.

1. EN and KO are readable in the installed font: no missing glyph boxes, raw localization keys, unexpected fallback English in KO labels, clipped titles or overlapping rows. Numeric AgentId names are expected where the existing API has no names.
2. Top view shows selected parent, position/state/target, three needs, portrait and carry/cold rows as applicable. Panel remains within620px; scrollbar does not obscure content or world controls.
3. Temperament view shows all four localized labels and bars with exact getter permille values/max1000. No stale prior-relative values remain after the scripted family clicks.
4. Family view shows localized sex, partner/mother/father rows, living-child count and up to five sorted living children. Verify agent IDs against actual census; absent/dead parents show localized None and disabled buttons. Additional children beyond five affect count, not extra fabricated buttons. Ensure bottom child row is not cut off.
5. The same parent is restored in both languages and every view; click records select actual partner/child entity bits. If manual pointer behavior is required, run a separately authorized bounded interactive check; signal-click records cannot certify pointer hit rectangles.
6. Optional rings absent means DROPPED/NOT IMPLEMENTED, never FAIL for a mandatory parent capture and never PASS for optional ring acceptance. If present, inspect subdued gold1px geometry relative to selection, only partner/living children, clear visibility without UI occlusion. Closing/reset/death lifecycle requires the separate optional fake/runtime tests.

If a family view still cannot show all required rows inside620px, retain failure and add actual intermediate scroll captures in a new attempt; do not declare readability from metadata. If no real partnered parent exists, preserve the failure rather than inventing links. No PNGs, real click results, visual verdict or runtime acceptance exist in this preparation directory yet.

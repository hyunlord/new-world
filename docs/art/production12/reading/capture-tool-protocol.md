# P12 life capture tooling protocol

Prepared before Part3 captures. Run only after the leader declares the global source freeze. Parser/unit checks are not runtime or reading acceptance.

`P12_LIFE_BUILD_PROFILE=debug P12_BUILD_DETAILS='dev opt2' python3 tools/art_pipeline/run_life_captures_p12.py CHECKOUT NEWDIR`

The new runner serializes each window through with_window_lock.sh, then waits inside that lock while the foreign .mac_quiet.lock exists (30second polls, complete wait/launch bounded at1800seconds), and preserves launch logs, working-tree patch, HEAD, bridge hashes, runner/script hashes, complete reference JSON and failed output. A source/HEAD change aborts the attempt. The reader directory contains anonymous PNG copies and instructions only; private-reader-map.json and all reference-data.json files remain outside it. Native bytes are copied without resize or re-render.

## Fixed worlds and subjects

- live: seed20261010 tick85080; lowest living AgentId with a found dead parent, then that actual parent in record mode.
- multi: seed19400 tick121080; one subject for every DepartCause actually in the complete chronicle, absent causes explicitly recorded; one actual stage-out porter resolved from entity bits to current AgentId. Capture porter Life EN/KO, then road-centred Z4/Z1.
- estranged: seed19414 tick34680; require actual kind19 quarrel and kind20 estrangement, capture an actual participant's Life EN/KO and village view Z1.
- crowd1/2/3: reproduce the three independently proven Part2 natural scenes, preserving originals. Respectively seed20261010/t41880/AgentId1/centre(22,20); seed20261018/t85080/AgentId29/centre(39,35); seed20261010/t42180/AgentId30/centre(24,21). Rediscover actual entity bits, require live selected person and visible icon, capture EN/KO. Fresh body/icon rectangles and full sprite draw order are collected on the screenshot frame; Python independently recalculates25-within10tiles and zero coverage before accepting a run.

All worlds reset freshly and verify actual seed, tick, ticks_per_day1440 and offset480. No simulation ticks advance after the fixed target. No sprite rearrangement, synthetic people, stress worlds or history injection.

## Text and evidence

From attempt 2, each frozen-world reference also stores raw tallies, settlement, building, activity and society snapshots through ReadRoute. Living captured subjects include actual entity bits and get_agent_detail when their AgentId resolves in the live agent snapshot. These additions supply fresh private evidence for HUD and activity claims; attempt 1 artifacts remain unchanged.

For each person and language, open actual inspector, expand Life before capture, and traverse every scroll page with48pixel overlap until the actual scrollbar end. Store complete Life text, actual record mode, porter-status visibility/text, page positions and person/history getters. Chronicle panel uses its real get_chronicle_since cursor and real formatter; the harness invalidates only its per-tick fetch guard to drain any backlog at the frozen tick. Drain count/cursor/total and complete displayed text are recorded. This capture-only pagination does not mutate history or simulation ticks. All panel pages are saved; the full underlying chronicle and referenced person records remain in private raw evidence even though the production feed retains its latest60 notable entries.

Each screenshot records village-layer metadata, including relation lines, and road screenshots record actual porter body metadata, icon rectangle/id/scale/alpha. Hash checks cover every GDScript, scene, shader, compiled locale JSON, font, atlas JSON/PNG, project.godot, GDExtension descriptor and both available bridge paths. Full hashes are rechecked before each image.

Fresh reading begins only after capture completion. Give the reader anonymous images/instructions only; freeze all factual/inference statements and three crowd-activity answers before comparing private histories. Report unreadable text, raw keys, percent/joiner artifacts, split names and wrong statements literally. No visual or semantic pass is inferred from successful tooling.

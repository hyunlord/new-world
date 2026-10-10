# Production decisions

- Start UTC:2026-10-10T11:06:37Z; twelve-hour cutoff:2026-10-10T23:06:37Z.
- PROD_BASE_SHA:0941e1b76ee250a017295fe923f7c0841dafaaa7. Direction merge:b5ead380c3eed047f6689224763d3713f7e5b02c, clean merge.
- Only the fresh /Users/rexxa/work/new-world-art-prod clone is modified. WIP gets each Part; green only advances after full gate. No simulation/bridge changes, no harness hooks.
- D3 throughout; muted D3 ground only. No D1/D2 assets installed.
- Image budget60 including failures; checkpoints20/40. Final calls:42; see IMAGE_BUDGET.json. Windowed runs always use shared lock; headless checks do not.
- Explicit task approval supersedes skill approval gates and generic lead/main reset workflow. Work follows the supplied Parts, preserving failed checks honestly.

## Part 0 baseline

Full baseline gate passed: cargo test 322s, Clippy14s, strict GDScript4s, bridge2s, smoke7s, smoke error scan0, Python1s. Results and logs: `gates/baseline/`. Baseline has no art scene tests yet. All renderer edits began only after this gate finished.

## Production rendering decisions

Under 200 villagers and 1,500 trees use pooled Sprite2D on one y-sorted layer. A y-sorted MultiMesh is reserved for a later 10k-agent renderer. Camera tween samples are snapped to the integer ladder, preserving nearest texels during the0.12s transition. The explicit art bible and BRIEF are the design system; web-specific Lighthouse/React rules do not apply to this native Godot surface.

## Generation observations

Calls1–4 requested1024×1024, but the built-in tool returned1254×1254. Raw files are preserved without resampling, and the converter measures their actual native grid. V_a1 andV_b1 accepted0/16 each; size and fidelity thresholds remain unchanged. After the first oversize sheet, prompts request less than half-cell figureheight and fewer flat colours; this is ordinary per-sheet size guidance, not the20-call checkpoint revision.

## Part2 gate and first look

Full gatepassed all13rows (cargo315s,Clippy1s,strict4s,bridge1s,smoke7s,Python2s,all6headlessarttests). Real finalplan10captures saved underp2, rawlosslessoutsidecommitteddocs. Firstlookandvisualverdictstored. The treepropertytest uses a directiontree through theproductionatlashelper; actualforestplacementverification remainsPart4/7. Reviewerrequested actualagentfeet andno-tint checks, added andpassed.

## Evidence storage

Productioncontact sheets preserve the specified6xlayout but publishJPEG85 to respect40MBdocsbudget; rawgeneratedPNGandper-cellacceptedPNG remainlossless. Integrationcheckpointkeeps onlynoonpreviewandverdict inGit, all10rawcaptures andentityJSON remainin `.harness/evidence/art-integrated/shots/final`. Fullfinalartevidencewillbepublishedseparately. Commit784fdc0a title saysflint, butitsacceptedcellisN2_05smallboulder; manifest/report areauthoritative.

## Final production and capture decisions

42 generations ended after required bounded character/structure retries and the checkpoint40 fallback route. Unique accepted generated cells13, procedural cells243; accepted-per-attempt26 includes13 preserved repeats. Every raw was1254x1254 despite1024 requested. No resampling or relaxed thresholds. Adult sleep now uses exact rotation of accepted narrow N-walk bodyB, with the view/body substitution declared. Direction-specific identity and two distinct adult bodies remain unmet.

Real art capture run at dc55c9a663320b57047c1ba50a6298e8164a06d0 produced92/92 frames. Documentation and static-test work was concurrent; render source matched that commit. Code gate and art acceptance remain separate. A5 min0.0548 and8unmeasurable occluded villagers fail; A11 blind labels saved before key reveal scored4/24 at both zooms. Ground chroma reduction cannot remove occlusion or provide missing activity silhouettes, so it was not used to chase the metric. Prescribed tree density was retained. The two bounded asset review rounds are complete.

Publishing entity metadata with whitespace removed saves5,106,833bytes without changing JSON values; raw pretty JSON and framebuffers remain under .harness. GIFs each preserve40frames and stay under8MB. Native Godot visual QA uses real framebuffer checker/y-sort/ramp measurements plus two independent read-only visual reviews; web DOM/Lighthouse checks do not apply.

## Publication boundary

Finalcodegate19/19passed at64db8021f9e9cae4b8aabb0725a5d047bcab4ef1. Finalpublicationcommitwillbeverified againwithGATE_OUTPUT_DIR pointingoutsidecommitteddocs, avoiding a self-modifying evidencecommitloop. Render/art/locale/sourcefiles areunchangedfromthe92-framecapturecommitdc55c9a6. Compatibilitymergedlatest simulationa8611e07cleanlyinlocalce6d9494, captured92frames, thenreturnedWIPanddeletedthetemporarybranchwithoutpushingit.

Rawmetadata forpublishedshots remainsunchanged; additional whitespace-onlycompactionofearlierP2/sheetJSON saves521,070bytes. Reportspacebudget remains40,000,000bytes. Independentper-Partwalltimers were notcollectedduringparallelwork; reportprovidescommitcheckpointwindows explicitlynotclaimedasexecutiondurations.

# H10 capture verdict: FAIL

H10 is **FAIL**: no acceptable seed19603 Z4 road-porter capture within three attempts. Actual EN/KO Z1 relation views and kind21 peace sentences succeeded, but do not compensate for the failed porter requirement. No further road retry is authorized by the three-strike rule.

## Evidence and visual inspection

The reporting agent personally inspected road EN and KO, relation EN and KO, and chronicle page000 EN and KO. These native PNGs and matching sidecars are retained in this directory. All are actual game_v26 seed19603 captures. They use the explicit **debug** build profile, loaded bridge SHA256 `ab9be4eda437835c79d0ff7d96f3a7addfd73a52c677bb4a241120e0069267c2`; they are not release-performance evidence.

| Target | Attempts | Result | Evidence |
|---|---:|---|---|
| Z4 porter on road, EN/KO |3|FAIL|Attempt1 runtime0 but visual failure; attempts2/3 reject covered candidates|
| Z1 relations in >=2 colours, EN/KO |1|PASS capture requirement|Visible orange solid, green solid and red dashed relations; sidecars record d9893a/7fb069/c0392b|
| Chronicle kind21, EN/KO |1|PASS capture requirement|Both page000 images visibly contain villages1/2 making peace at day86|

- [Road EN](porter_road_z4_en.png), [Road KO](porter_road_z4_ko.png): the intended selected porter155 is in the centre cluster. A mint SitByFire badge covers the intended porter-glyph region; adjacent bodies interfere with identifying its carrying pose. The broader scene contains other package-like glyphs, but those are not a substitute for the designated subject. Both languages fail the requested unambiguous porter evidence.
- [Relations EN](relations_z1_en.png), [Relations KO](relations_z1_ko.png): the world overview shows three village clusters linked by green and orange solid lines plus a red dashed estrangement line. Language-specific village labels are present. Thin lines and dense settlements remain visually modest, but multiple relation colours are directly visible.
- [Chronicle EN](chronicle_en_page_000.png): the visible line reads “Day 86 · Village 1 and Village 2 made peace.”
- [Chronicle KO](chronicle_ko_page_000.png): the visible line reads “86일째 · 1번 마을 · 2번 마을 화해함”. The actual raw kind21 reconciliation entry is retained in the external reference. The sentence is visible on page000, not merely present in an offscreen text buffer.

## Retry accounting

1. **Road1, tick122430, HEAD8ca6b93018b0ec0b795f8312c549197a97f851da:** runtime returned0 and `complete:true`, with two PNGs. Visual review rejected them. The earlier runtime TSV is retained literally; it is not an H10 pass.
2. **Road2, tick122450, HEAD3bee1316acdbcb891bda0be1385ba959bbb1e7f1:** stricter actual draw-order body/icon coverage rejected both outbound candidates. Agent155 had5 body blockers/1 icon blocker;177 had9/2. Startup decorative dust and real agent overlaps were retained in failure evidence.
3. **Road3, tick122479, HEAD75c2ea6a2cdfb085a4a4a90ceaf9fe257cda16a3:** final attempt after a frozen-render settling interval. Both candidates still failed conservative zero foreign coverage:155 had2 body blockers/1 icon blocker;177 had3/1. No accepted PNGs. The terminal log states `no outbound porter has an unobscured road position at requested tick`. Target stopped at3/3.

Relations and chronicle ran at tick122430/HEAD8ca6b930, runtime0, with2 and27 PNGs respectively. No fake bridge/history was substituted. Checks did not alter world positions or weaken coverage to obtain a pass.

## Artifact provenance

`raw-manifest.json` hashes72 external files, including every H10 original screenshot, sidecar, reference/failure JSON and launch log under `/Users/rexxa/work/art-production13b-evidence`. Heavy raw references remain external. Selected copies here are byte-identical to originals. Logs for all five runs are copied here. The manifest records absolute paths, byte sizes and SHA256; it does not claim those external files are committed.

The supplied source chronicle and final screenshots establish actual peace/relations evidence. H10 remains failed because its conjunction includes a readable porter glyph and carry pose on the road. Fact-reading scores are a separate process and are not asserted by this report.

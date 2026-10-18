# ROUND1 Pass B — visual fidelity and CJK precision

VERDICT: PASS
CONFIDENCE: HIGH for inspected text/layout; MEDIUM for low-zoom semantic readability.

Scope: native Godot 1920×1080 captures, checkout HEAD `33b8f7c7`. Read-only screenshot/source review using the visual-qa skill. No user mock was supplied; intentionally changed UI is not required to pixel-match the older fixture. No GOAL sections 3–7, earlier review answers, private reader map, or life-reader outputs were consulted. No Godot/cargo execution or source edits.

## Findings

BLOCKING: none in the inspected scope.

OPTIONAL: The red estrangement midpoint caption in `estranged/estranged_villages_z1.png` (approximately x936–1015, y330–346) is low contrast against green ground and visually intersects a passerby at x970. The dashed relationship line itself remains traceable between the two settlements, and the caption is localized rather than a raw key. Consider a small contrasting backing/outline and displacement around actors for easier reading. Source: `scripts/ui/settlement_relations_p12.gd` draws the relation layer/caption. This is a readability improvement, not evidence that the relationship line or localization is absent.

## Direct pixel observations

- First and final Life/record pages: names are visibly bold and underlined in event prose; no inspected proper name splits across lines. English names including Timiga, Mabito, Hugade, Bimuo and Korean equivalents remain whole. Korean 티/디 distinctions remain visually present; no tofu or visible word-joiner boxes were observed.
- Ordinary Korean body text wraps by syllable in the narrow inspector: `태어/남` in porter Life, `다/툼` in quarreller Life, `세상/을` in the death record. These are readable body-word wraps, not split names or missing glyphs. Name-only continuation lines such as 토토수 or 가와 remain intact; they are not one-syllable name orphans.
- Final scroll captures show complete final entries with room above the panel's lower edge: death at 23 days, living parent's child at day 49, departure subjects' elder events, porter day-85 aid departure, quarrel day 23, and final Chronicle rows. Partially visible preceding content at the top of a scrolled page is normal viewport clipping; it does not truncate the final entry.
- First-page roles, statuses, cause of death, family labels and porter cargo/status are localized. No raw localization keys, `%s`/`%d` placeholders, or BBCode appear in inspected pixels.
- Porter Life shows food ×10 to Village 1 / 식량 10 → 1번 마을 and a complete final route entry. `porter_road_z4.png` shows distinct tan pack markers above the two carriers, plus carried goods. At z1 the pack markers remain small tan vertical marks; the green inter-settlement line is continuous and visible. Low zoom does not preserve the detail of the zoom-4 pack art, which is expected.
- Estranged low zoom has a red dashed current relation line and Korean caption; English/Korean Chronicle first pages explicitly show the villages falling out. The line and the historic event are separate visible surfaces.
- Crowd1/2/3 selected figures and their attached icons are visible in the proof rectangles. Nearby crowd overlap exists elsewhere, so this is not a claim of zero overlap throughout a settlement or of reliable human activity recognition.
- Native-original reinspection was used for ambiguous small digits in Chronicle and the estrangement caption. No dropped date digits were confirmed; scaled image presentation must not be mistaken for a source-rendering defect.

## Objective evidence consumed (not substituted for pixels)

`p2/font-attempt1/font-result.json`: actual Life size 24; regular/bold plain-vs-joined differing pixels 0/0; 티-vs-디 differing pixels 24/32; passed true. Directly viewed all eight corresponding plain/joined/ti/di raster samples. They support invisible joiners and distinct glyphs, not a blanket claim about every string.

`p2/layout-attempt2/{en,ko}-{top,bottom}.png`: all four directly viewed. Earlier 360-pixel-wide inspector framing and readable end entries are preserved in the live scenarios, now with actual world and portrait content.

`docs/art/production12/perf/declump-kernel/pixel-comparison.json`: P1–P6 each report control/candidate differing pixels 0, max channel 0, identical three-way RGBA hashes, bit reproducible true. Capture frames are 130 except P3=121; state-change frames 10 except P3=1. This is kernel identity evidence for those framings, not a pixel-match demand on the intentional new UI. Numeric-clean JSON was also inspected (Godot 4.6 official, non-double build and explicit numeric bit cases); no performance or fresh-run claim is inferred.

All six fresh `crowd*-crowd-proof.json` files report zero selected-body and selected-icon covered fractions, no exclusion, native scale 1/no resampling, and `zero_coverage_crowd_proven=true`:

| Scene | Nearby within 10 tiles | Native selected proof box |
|---|---:|---|
| crowd1 EN/KO | 25 | [928,268,1112,452] |
| crowd2 EN/KO | 37 | [998,256,1198,456] |
| crowd3 EN/KO | 25 | [844,372,1048,576] |

The geometry implementation measures foreground foreign draw rectangles relative to the selected holder, excluding its own attached art (`prepare_blind_crops_p12.py`, `row_geometry`/`crowd_proof`). This explains why a crowded scene can legitimately have a zero-coverage selected holder. Each proof retains `activity_reading: NOT_VERIFIED`; this review does not replace that separate gate.

## Images inspected

All paths below are relative to `/Users/rexxa/work/art-production12-evidence/`.

- `p3/live-attempt1/live/living_dead_parent_en_page_{000,003}.png`
- `p3/live-attempt1/live/living_dead_parent_ko_page_{000,003}.png`
- `p3/live-attempt1/live/dead_parent_en_page_{000,003}.png`
- `p3/live-attempt1/live/dead_parent_ko_page_{000,002}.png`
- `p3/live-attempt1/live/chronicle_en_page_{000,014}.png`
- `p3/live-attempt1/live/chronicle_ko_page_{000,011}.png`
- `p3/live-attempt1/multi/depart_cause_1_en_page_{000,004}.png`
- `p3/live-attempt1/multi/depart_cause_1_ko_page_{000,004}.png`
- `p3/live-attempt1/multi/depart_cause_2_en_page_{000,004}.png`
- `p3/live-attempt1/multi/depart_cause_2_ko_page_{000,003}.png`
- `p3/live-attempt1/multi/porter_life_en_page_{000,002}.png`
- `p3/live-attempt1/multi/porter_life_ko_page_{000,002}.png`
- `p3/live-attempt1/multi/chronicle_en_page_{000,014}.png`
- `p3/live-attempt1/multi/chronicle_ko_page_{000,011}.png`
- `p3/live-attempt1/multi/porter_road_{z4,z1}.png`
- `p3/live-attempt1/estranged/quarreller_life_en_page_{000,003}.png`
- `p3/live-attempt1/estranged/quarreller_life_ko_page_{000,003}.png`
- `p3/live-attempt1/estranged/chronicle_en_page_{000,014}.png`
- `p3/live-attempt1/estranged/chronicle_ko_page_{000,013}.png`
- `p3/live-attempt1/estranged/estranged_villages_z1.png`
- `p3/live-attempt1/crowd{1,2,3}/crowd{1,2,3}_{en,ko}.png` (matching directory/file number; six images)
- `p2/layout-attempt2/{en,ko}-{top,bottom}.png`
- `p2/font-attempt1/{regular,bold}-{plain,joined,ti,di}.png`

Total: 57 distinct PNGs directly opened with image viewing. Intermediate scroll pages were not exhaustively inspected. This PASS applies to the requested first/end visual and CJK check, not the blind-reader outcome, interactions, all possible strings, resizing, or runtime/performance gates.

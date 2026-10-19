# Part 3 frozen factual reading score

36/36 PNGs visually inspected and SHA-verified against frozen mapping; answer SHA matches freeze. 692 atomic text claims: 624 independently correct, 0 incorrect, 68 screen-match-only / not independently verified.

Names: 336 token readings, 320 exact, 16 transcription errors. Names are not added to the factual denominator. Seven uncertain picture judgments receive no confident activity-recognition credit.

Counting: One claimed scalar or relation field per event; day and event kind separate; coordinate pair one field. Each repeated screen occurrence counts. Parent count includes explicit named parent pairs in Names (spellings scored separately). Names count each full token transcription occurrence, including repeated roles and legible preceding fragments. Picture inferences/artifact observations are retained separately and excluded. No offscreen events credited. Berry-bush resource context, glyph colors/counts, printed-value absence, construction, berries and daytime are screen-only when independent raw representation is absent. Clipped-cause observations are screen-only, not assumed cause claims.

Raw global chronicle independently matches every selected life-event tick/kind/subject. Day32 quarrel direction is reversed in the global event: selected23 is the other participant; the counterpart remains village2. Event ledger and per-field pointers are in part3-score.json.

| Image | Facts verified | Screen-only | Names exact | Name errors |
|---|---:|---:|---:|---:|
| image_0001.png | 23 | 3 | 1 | 0 |
| image_0002.png | 13 | 5 | 5 | 0 |
| image_0003.png | 18 | 2 | 5 | 0 |
| image_0004.png | 16 | 2 | 5 | 0 |
| image_0005.png | 20 | 3 | 1 | 0 |
| image_0006.png | 6 | 5 | 5 | 0 |
| image_0007.png | 20 | 2 | 6 | 0 |
| image_0008.png | 16 | 9 | 0 | 0 |
| image_0009.png | 21 | 2 | 7 | 1 |
| image_0010.png | 18 | 2 | 9 | 1 |
| image_0011.png | 14 | 2 | 9 | 1 |
| image_0012.png | 18 | 2 | 9 | 1 |
| image_0013.png | 15 | 1 | 10 | 0 |
| image_0014.png | 16 | 2 | 10 | 2 |
| image_0015.png | 16 | 2 | 9 | 3 |
| image_0016.png | 20 | 0 | 7 | 0 |
| image_0017.png | 17 | 1 | 9 | 0 |
| image_0018.png | 18 | 0 | 10 | 0 |
| image_0019.png | 15 | 1 | 9 | 1 |
| image_0020.png | 13 | 1 | 10 | 1 |
| image_0021.png | 15 | 1 | 11 | 1 |
| image_0022.png | 19 | 1 | 11 | 1 |
| image_0023.png | 17 | 1 | 10 | 1 |
| image_0024.png | 20 | 3 | 10 | 0 |
| image_0025.png | 21 | 1 | 12 | 0 |
| image_0026.png | 17 | 2 | 12 | 0 |
| image_0027.png | 17 | 1 | 11 | 0 |
| image_0028.png | 16 | 2 | 13 | 1 |
| image_0029.png | 16 | 2 | 11 | 1 |
| image_0030.png | 24 | 0 | 8 | 0 |
| image_0031.png | 17 | 1 | 11 | 0 |
| image_0032.png | 21 | 1 | 12 | 0 |
| image_0033.png | 17 | 1 | 14 | 0 |
| image_0034.png | 18 | 2 | 13 | 0 |
| image_0035.png | 19 | 1 | 12 | 0 |
| image_0036.png | 17 | 1 | 13 | 0 |

Name errors (reader → rendered spelling):

- image_0009.png: Ntiso → Nitiso (person 0).
- image_0010.png: Etuha → Etuba (person 10).
- image_0011.png: Oedo → Dedo (person 89).
- image_0012.png: Oedo → Dedo (person 89).
- image_0014.png: Buyohi → Buyobi (person 35).
- image_0014.png: Neba → Nieba (person 51).
- image_0015.png: Neba → Nieba (person 51).
- image_0015.png: Budoho → Bubono (person 84).
- image_0015.png: Baya → Baga (person 64).
- image_0019.png: Ntiso → Nitiso (person 0).
- image_0020.png: Baya → Baga (person 64).
- image_0021.png: Ntiso → Nitiso (person 0).
- image_0022.png: Dehoha → Dehona (person 3).
- image_0023.png: Dehoha → Dehona (person 3).
- image_0028.png: 부요히 → 부요비 (person 35).
- image_0029.png: 부도호 → 부보노 (person 84).

Limitations:
- Raw persons contain no independent rendered name strings. Name transcription compares the viewed glyphs to sidecar person-ID-linked rendered spelling; this is a legibility score, not independent name-generation validation.
- Picture-only selected activity uncertain in all seven Life screens; no confident recognition credit.
- Render-only color, resource wording and presentation observations cannot establish independent raw world truth.
- Repeated screenshots are correlated, not independent participants.
- Family parent identity is raw-verified; name spelling scored separately.

Sources:
- Frozen answer: `/Users/rexxa/work/art-production13b-evidence/fact-reader-answers-part3.md` SHA256 `486b6a4700ab4adc51244d7634348a41238a1b80f68d1d3e797f56e88602319f`.
- Freeze: `/Users/rexxa/work/art-production13b-evidence/fact-reader-freeze-part3.json` SHA256 `4414027c7b5857198f4698db86761bb922bb048fbe3bf1daf12182505530ebf4`.
- Mapping: `/Users/rexxa/work/art-production13b-evidence/life-reader-part3-private-map.json` SHA256 `8da09bd1c92576943d653058fe2052c33d49e97d4e5dd2a1d6d8a47b418078c1`.
- Independent raw reference: `/Users/rexxa/work/art-production13b-evidence/life-estranged-attempt3/reference-data.json` SHA256 `fab34ec990cd7332f24d47663559d83b40db77e134730ae510a1b323d697c7e7`.
- All individual PNG and sidecar hashes and exact frozen answer excerpts are retained in JSON.

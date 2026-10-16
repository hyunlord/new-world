# Part 5 protocol review, round 1

HEAD reviewed: `90045db04807850be0c5c5c2cb792251c2575da8`.
Read-only source/evidence review; no Godot, cargo, runtime, gates or code edits. Only this report and sanitized reader provenance written externally.

Overall: PASS for truthful final reporting and retained two-round evidence. H7 remains NOT_VERIFIED; C diagnostic recognition FAIL. No release-blocking misstatement found in the reviewed activity report. One bounded provenance limitation below.

## Independently checked evidence

- Archive SHA256: `edb516330eaf8f36c85c120369a5abf915a5cb6a1c155ed9eba38aaec4805fad`; manifest SHA256 `7811f307ed7b3581272ec586f99039749274841db7e81a79854d57d6a931cf22`. All 268 members match manifest size/hash and current original source bytes. This is an actual ZIP byte check, not reliance on `roundtrip_verified`.
- Recomputed holder selection using the eight ordered C sidecars, arrived condition, sorted AgentId, rectangle-union coverage and greater draw ranks. All 129 exclusion rows match exactly. Selection matches all 12 key entries and counts `[1,2,2,2,1,2,2]`; no duplicate padding. All eight C sidecars contain schema2 complete world-sprite ordering, 516–732 draw rectangles, including tree and descendant/world entries. Completeness of actual renderer enumeration is a source/capture contract, not a new runtime proof here.
- Five supplementary cameras match the lowest arrived AgentId of their specified kind: 32,53,96,108,59. Camera positions equal raw snapshot tile coordinates times16 plus(456,36), consistent with `scripts/test/art/capture_activities_p11.gd:83–96`. All seven actual supplementary launch preregistration hashes match launch records, including the two failures; preregistrations precede launches and crops. Final valid supplemental launch finished12:04:53Z. Terminal shortage authorization12:06:20Z precedes extraction.
- Exclusion freeze1792152380484839000ns precedes actual earliest crop mtime1792152380495417200ns by10.58ms. All selection-history freezes precede it. Re-extracted all12 crops from original source PNGs; every pixel and dimension equals the numbered PNG using ceil(max(body width,height)+16), body center,4x nearest-neighbor. No enlargement to include icons.
- Pose artifact contains15 captures,503 checked holders,0 mismatches. Rejected attempts1/2 remain in ZIP. This review does not claim a fresh runtime pose run.
- D preregistration12:07:23.280862Z follows C1 freeze12:07:10.490330Z. All four fixed11/14 supplemental cases retained. Six selected pairs have maximum2 per(seed,village); all six top counts strictly exceed runners-up, and true kinds match keys. The original tied day30 15:00 summary remains excluded.
- All four answer-file SHA256 values match score records; each answer's image hash matches its actual numbered PNG. Recomputed scores: C1=0/12,C2=1/12,D1=2/6,D2=5/6. Full C cannot pass the unchanged14 requirement; five fully sampled kinds remain0/2. D2 PASS does not pass H7.
- Round1 C freeze12:07:10.490330Z < D freeze12:24:59.649287Z < reveal12:25:32.068021Z. Round2 C freeze12:28:31.917495Z < D freeze12:29:55.190911Z < reveal12:30:27.998189Z. D preparation binds the respective frozen C hash.

## Native reader provenance

`protocol-reader-provenance.json` preserves only relevant tool-call inputs and sanitized spawn metadata, with original source SHA256 and line numbers. Full session logs and image payloads are not copied.

- Root log lines3306/3634 explicitly use `fork_turns:"none"` for two distinct reader agents.
- Reader1 log lines11,30,44,57,66 show names/numbered images, C answer write, then D names/images and D answer write. No key, sidecar, GOAL or icon-sheet read occurs in any recorded tool call. Batched image viewing is visible and consistent with the retained attribution concern.
- Reader2 log lines11–124 show individual labeled C images and immediate answer recording, ending with C freeze; lines138–199 show D images/answers afterward. No key, sidecar, GOAL or icon-sheet reads occur. The second round uses the same unchanged image hashes.
- Round1 D03/04 answer descriptions mismatch source summary counts; round2's one-image-at-a-time protocol is evidenced in actual tool calls. Round1 failure was preserved, not rewritten. Only two reader rounds were found in the specified evidence.

## Findings / limits

1. **LOW / NOT_VERIFIED — plaintext reviewer task prompt cannot be independently audited from encrypted logs.** Root session lines3306/3634 and reader session line10 store the task message encrypted. `fork_turns:none`, tool-access isolation and chronological answer writes are verified; complete absence of key information inside the encrypted initial/follow-up prompt cannot be asserted by this reviewer. No observed evidence of leakage. This is a bounded provenance limit, not a reason to reinterpret H7 as PASS or erase measured scores.
2. **Known acceptance shortfall / NOT_VERIFIED — full C12/14.** `docs/art/production11/GOAL.md:94`, `activity/blind-score.md:3`, `activity/verification.md:3`: two mandated samples are absent. Current final wording accurately preserves original bar and reports failure of available recognition separately. No correction needed.

Decision: activity reporting PASS; complete blind-C acceptance NOT_VERIFIED; H7 NOT_VERIFIED; D round1 FAIL and round2 PASS. No runtime work or third round requested.

# Merged walker identity evidence

Compound identity **PASS**, comparator exit 0. The terminal-session record is preserved in `TERMINAL-SESSION.txt`.

The compound report is `comparison-attempt1/walker-identity.json`. Inherited reports retain their `reference_integrity_only` scope. This archive records identity evidence only; it does not establish performance, visual acceptance, or installation approval. Consult the actual report for errors and limitations.

All raw PNGs, sidecars, logs, launch tables, provenance and comparison outputs are copied byte-for-byte. Ten complete raw JSONL files remain external: `SHA256-MANIFEST.json` records their absolute paths, SHA256 and byte lengths. Every copied file is likewise hashed. The archive helper verified source stability and copy hashes; it did not rerun captures or comparators.

## Exact measured result

- Session 53070 terminated with exit 0; all 28 launches completed. Compound comparator exit 0, `passed: true`, `errors: []`.
- P1–P6 each have **0 differing full-frame RGBA pixels, maximum channel difference 0**, for both A/A′ and A/B. No masks, exclusions or tolerance were introduced.
- Running W0, W200 and W-multi each contain 600 frames per polarity; W500_FROZEN and W500_2X each contain 240. All five off/on complete JSONL pairs are byte-identical and pass inherited schedule/population/camera/render checks.
- Compound validation confirms actual walker false/false/true across the six A/A′/B framings and false/true across the five running off/on scenarios, declump kernel ON, write skip OFF. Coverage is 18 capture-time still stamps, 4,560 running-frame stamps and 10 final running stamps. Still collectors do not emit per-frame candidate stamps.
- Launch provenance and before/inside/after snapshots pass unchanged-source/build checks across all five groups. P3 camera processing is disabled and camera pause is false in all three still captures.
- The leader visually inspected merged `six/B/P3.png`. This narrow inspection is recorded as reported by the leader; it is not broad visual acceptance of the run or product.

Capture HEAD: `f5c8951e0a0a5861defe632cbd2a088dac71f692`.
Bridge SHA256: `30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042`.
Source-manifest SHA256: `0a9091c208ba409583e8c847d241265f7fdaccf53ff1a1411075de0c371f74de`.

Retention: 276 raw file records, comprising 266 verified archive copies and 10 complete external JSONL references. The archive additionally contains this README, terminal-session record and manifest. Earlier attempts remain separate and unchanged.

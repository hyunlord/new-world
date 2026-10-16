# Person names (before first capture)

Names are proper nouns, not localization keys. The double hash, 46 syllables,
2/3 syllable rule and fixed EN/KO blocklists follow GOAL 5.1. Both languages
are checked together so changing locale never changes the underlying syllables.
No blocklist extension was made.

## Termination decision

Agent 282 hashes to indices [25, 36, 18]: `sibamu` / `시바무`. Cycling only
the final syllable cannot remove the blocked prefix `시바`. We first try all
46 final syllables, exactly as prescribed. After exhausting them we carry
into the previous syllable (base 46), then resume the same final-syllable cycle.
The finite 2/3-syllable space contains clean names, so the loop terminates.
This narrow exception was decided before any Part 1 capture. Tests include
agent 282 and all IDs 0..9999, both scripts, determinism and collision shares.

## Locale keys

All 36 section-10 keys were checked against the existing Fluent sources
before adding them. Existing UI_CHRONICLE_TITLE, UI_LIFE_EVENTS, UI_DECEASED,
UI_CAUSE_OF_DEATH, UI_FAMILY, PANEL_FAMILY_SPOUSE, UI_MOTHER and UI_FATHER
are reused unchanged. Compiled maps and stable next registry IDs are generated
by tools/localization_compile.py against the external preparation root.

## Chronicle feed backlog

The world feed polls from the delivered `next` cursor once per simulation tick,
including while hidden, and keeps the last 60 notable entries consumed. The
bridge caps one page at 512 entries. A large artificial `advance_ticks` jump
can therefore leave `ChroniclePanel._next < _total`; if paused, that backlog
cannot drain without violating the once-per-tick rule. The panel does not
claim to be the latest world history while stalled. Capture tooling advances
real simulation ticks, recording the exact final tick, until `next == total`.
The fake feed test uses 700 entries: 120 fixed-tick frames read one page only,
then the next actual tick completes the feed. This limitation is retained in
the final report rather than hidden by extra calls or a changed bridge API.

A living person's cleared `Kin.partner` falls back to their latest Paired or
Widowed chronicle entry, so the deceased spouse remains a named record link.
A current live partner takes precedence; later pairing entries override older
widowhood entries when reconstructing historical records.

# Mandatory-coverage continuation

**Result: PARTIAL.** The bounded continuation adds21 accepted frames without replacing any of the383 previously packaged assets. The new404-asset atlas passes mechanical validation. Required wardrobe coverage is256/612, with356missing keys; the frozen24-person request board now has22complete and2incomplete composites.

The historical checkpoint is `caf2557916c2fa35be47d492d051d87116204343`. Its registry, atlas, board and cost ledgers are preserved under external `continuation_accounting/baseline/` and in Git history. The original objective, deadline, acceptance thresholds and blind results have not changed.

| Lane | Calls | Returned | Tool refusals | Mechanical passes | Final accepted |
|---|---:|---:|---:|---:|---:|
| Adult male east-idle board hair |6|6|0|1|1|
| Child hair and beard |49|47|2|2|2|
| Elder hair and beard |55|55|0|21|13|
| Elder tools, excluding spear |32|32|0|8|5|
| Total |142|140|2|32|21|

The eleven mechanically valid but visually rejected candidates remain outside the final atlas. Tiny unreadable styles, detached tool parts and lost material cues are not promoted merely because their pixels pass the checker. All140returned originals match their archived bytes. Each previously missing child/elder key received one candidate; the three board-hair keys received two each. No additional retry wave was run in this continuation.

## Preservation and remaining gaps

- All383previous asset entries and source hashes are unchanged.
- All12frozen scene/A-B/crop files and all6random sample images/recompositions are unchanged.
- The board attributes retain SHA256 `d2153a372f56e9859211be52c7fd39b2f4a4b330519c32bffaabb5613436dc9c`.
- Board request4 is now complete. Request5 still lacks `hair/long_tied/adult_m/east/idle` and `tool/spear/adult_m/east/idle`; request6 still lacks `hair/braided/adult_m/east/idle`.
- Scene scores remain3/10and3/10; variation remains10/24. The one permitted scene revision is exhausted. No new blind score or human acceptance is claimed.
- All24rendered pixel sets remain distinct, but22complete composites do not meet the24-composite requirement.

The existing spear originals were also tested under378preregistered global scale/translation transforms. None passed body preservation; no local pixel repair or acceptance relaxation was applied. Explicit generation refusals remain recorded, and no further spear generation was attempted.

## Reproduction and limits

External root: `/Users/rexxa/work/arttrial-data/continuation_accounting/`. `integration/integrate.py` reads immutable historical snapshots, rejects accepted-key collisions and writes a fresh timestamped output directory. It does not rebuild or overwrite the frozen scene. `CONTINUATION_VERIFICATION.json` records the final manifest hashes, preservation checks and atlas validation; `ATLAS_PROVENANCE.json` records exact source-to-atlas pixels, boxes and anchors.

The updated coverage board was inspected for labels, omission borders and header accuracy. Its pixels match the parent-reviewed provisional board. This verifies disclosure and layout, not improved age/sex/tool readability. The additional hair/beard selections are internal visual judgments at a very small scale.

The shipped pipeline again passes65tests, Ruff and type checking with zero errors or warnings. External integration helpers pass3regression tests and strict type checking. Separate accounting-tool warnings are disclosed in `VALIDATION.md`; they are not hidden within the shipped pipeline result. Current and historical costs are distinguished in `COST.md`.

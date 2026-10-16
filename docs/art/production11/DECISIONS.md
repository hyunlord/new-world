# Art Pass 11 decisions

- GOAL.md is preserved byte-for-byte and supersedes session-start reset/lead/main instructions. No hooks; git mutations use core.hooksPath=/dev/null. No harness pipeline; the explicitly required art gate may invoke its strict-check helper.
- Execute the supplied decision-complete plan directly; no approval handoff or new planning workflow. Independent bounded implementation lanes may prepare tools and later UI, but Parts are integrated and published in order.
- Keep earlier evidence immutable; only the authorized simulation merge brings simulation-owned changes.
- Names are proper nouns; they are not localization keys.

# Person activity tooltip

The new mouse-ignoring overlay uses actual drawn-agent picking and resolves the current visible activity icon. After 0.4 seconds over the same person it shows PersonNames plus the existing localized activity label, using the single new UI_TOOLTIP_PERSON_ACTIVITY_FMT key in the five required localization files. No selection operation is called. Mouse-out, blocking controls, absent activity, stale icon state, hidden UI, world reset, focus loss and locale changes clear the tooltip.

Audited CLI-Anything headless test: `/Users/rexxa/work/art-production13-evidence/tooltip/unit-attempt2.json`, returncode 0, stderr empty, explicit `PERSON_ACTIVITY_TOOLTIP_P13 boundary_failures=0 real_input=NOT_VERIFIED`.

The fixture loads real Main/WorldSim and checks renderer hit geometry, the 0.4-second boundary, English/Korean strings and the clearing cases. It directly drives the helper; it is not the real-input harness. First attempt's unqualified Node notification constant caused a test parse error; that failed transcript is retained and only the test qualification was corrected.

Real mouse-input acceptance and visible placement remain NOT_VERIFIED because the user prohibited GUI windows. Pointer-active picking cost is NOT_MEASURED; the no-motion path exits before picking. These limitations prevent an H9 pass.

The p13 real-input harness now preserves its original13 successful interaction steps and appends four tooltip steps: before-delay hidden, after-delay localized text visible, mouse-out hidden, and no-activity hidden. It uses the existing InputEvent route, live day30 activity targets, actual viewport pointer geometry and screenshot-time assertions; it never calls the tooltip helper directly. Only arrangement positions/visibility are controlled, as in the original overlap test. Selection must remain unchanged. CLI-Anything headless syntax validation returned valid=true/errors empty; the display-only test itself remains NOT_RUN under the GUI prohibition.

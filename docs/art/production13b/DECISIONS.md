# Autonomous decisions

- The supplied Art Pass13b brief overrides generic lead/main reset and post-work instructions. Work only on art WIP and explicitly required side branch/worktrees. Hooks disabled per git invocation. No simulation merge.
- Launch HEAD and remote WIP both 8c1b18be8b05bfeaa062a937526fe4d9c2f53414; remote publish 57ae7ac40abde7d24369606c9250d3e06e0f18d6; simulation remote unchanged 354e4a13b21a20b28c65d84fab103c40b5143fa0.
- Neither production13/OWNER_FEEDBACK.md nor production13b/OWNER_FEEDBACK.md exists at launch. No feedback items to apply.
- Preserve local .gitignore, untracked .ignore and pre-existing production13/perf/attempts.lock; stage explicit new paths only.
- Calibrate before baseline gate: inherited calibration rule explicitly requires 15 probes before any Godot session. Use unchanged cpu_probe_p13.py with new output path, 20-second spacing.
- Existing gate uses windowed real-input checks and headless unit checks. Preserve original unit coverage; separately rerun requested Life layout/scroll checks windowed. Do not treat headless as screen evidence.
- Planning skill consulted; supplied six-Part execution contract is already decision-complete and explicitly requires unattended execution, so no planning approval handoff.
- Four copied measurement tools change only wrapper/output routing; copied wrapper also defaults its attempt ledger to production13b so an omitted environment variable cannot mutate old evidence. Existing collectors, validators and CPU probe remain unchanged. Shell syntax and Python byte-compilation passed; exact source diff inspected.

## Input coordinate repair (Part1)
Preserve the baseline failing attempt. Tooltip and independent input witness now consume delivered motion coordinates, since required Input.parse_input_event does not move the OS cursor. No cursor warping, handler invocation or weaker visibility/text/delay/selection assertions. Gate entry forwards to p13b test copy. Windowed attempt2 passed17/17.

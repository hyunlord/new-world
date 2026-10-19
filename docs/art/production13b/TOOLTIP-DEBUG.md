# Tooltip input diagnosis

Baseline target attempt 1 failed, preserved in gates/p0. Hypotheses:
1. Pointer-source mismatch: confirmed in code and runtime. `_input` receives motion but discards position; `_process` polls viewport OS pointer outside window. Event coordinates drive all passing click steps.
2. UI obstruction: not established as cause; retain gui_get_hovered_control blocking, verify after event-coordinate fix.
3. Wrong fixture/entity or timing: direct hover tests pass; real-input target remains the decisive test, not assumed correct.

Fix: retain delivered InputEventMouseMotion position and use it for per-frame picking. Window exit/focus loss still clear the pointer and hover. No direct invocation or OS cursor warp.

Harness defect: its independent geometry assertion also polled OS coordinates. Copy to real_input_p13b.gd and retain gate entry as forwarder. Independent InputWitness now records received motion coordinates, geometry assertion uses that delivered point; viewport OS coordinates remain logged. All selection, expected entity, visible text and delay assertions remain. No injected expected coordinates are used as observed state.

Official API reference: https://docs.godotengine.org/en/stable/classes/class_input.html#class-input-method-parse-input-event

Validation: attempt2 windowed exit0, all17 steps pass, including all13 inherited steps and4 tooltip steps. Before-delay elapsed0.024799s; after-delay0.491345s with exact expected text; move-out and inactive target hidden. Screenshots visually inspected. Windowed boundary test exit0. Strict check passed. Failure strikes remain1/3.

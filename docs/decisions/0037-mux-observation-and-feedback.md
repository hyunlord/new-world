# ADR 0037 — MUX observations and feedback remain outside simulation authority

Status: accepted for Stage A implementation; runtime, visual and performance evidence remain separate.

## Context

The owner needs visible Mac controls, alerts and learning guidance while retaining command authority. UI polls can miss short interactions, hide intermediate automatic actions, or confuse daily demand with seasonal readiness. Preferences can also contaminate automated scene fixtures.

## Decision

Camera gestures and on-screen controls call the existing camera APIs. The shared controls catalogue supplies help and tooltips; alert, goal and lesson catalogues select named observation predicates. GDScript observes bridge snapshots and sends ordinary commands, never writes simulation resources. Expensive facts and minimap refreshes run at most twice per second; short panel-opening interactions are latched from actual visibility events rather than inferred from a later poll.

Winter preparation compares global usable firewood with the sum of settlement daily requirements multiplied by the complete upcoming winter or the inclusive remaining winter days. Unknown or nonpositive requirements cannot award a preparation achievement. Alert causes retain their real settlement locations while sharing the aggregate budget. The observed fact does not grant fuel or promise survival.

Each accepted Steward action already has an authoritative Chronicle sequence. A separate incremental UI cursor queues every new action and preserves order and distinct repeated-rule actions. Player feedback is immediate; an interrupted background notification resumes afterwards. Queue delivery reads labels from the same supplied state as its action sequence, avoiding stale cross-world formatter state. Missing labels have an explicit localized unknown fallback. Existing Chronicle panel navigation remains independent.

Attach and successful world replacement establish notification baselines synchronously. SimReads emits a successful replacement receipt before returning to its caller, so an immediate subsequent tick cannot be mistaken for old save history. The HUD counts matching deferred native reset callbacks to avoid a second baseline; failed loads do not emit replacement receipts. Direct native callers retain the native-signal fallback, without claiming the synchronous guarantee of the ordinary UI mutation path. Loading a save does not replay its old notification backlog.

User preferences remain in user://settings.cfg and are separate from saved simulation state. Explicit test paths take precedence over WORLDSIM_TEST_SETTINGS_DIR; the environment opt-in redirects only the default store to an external per-process path. Normal game defaults are unchanged. Historical renderer fixtures explicitly issue OFF commands, while smoke and new-game tests retain production defaults. This is not global user:// isolation.

## Alternatives considered

- Recompute all observations every frame: rejected because UI reads and minimap work have explicit budgets.
- Sample panel visibility at the bulk-fact rate: rejected because real open-close interactions can occur between samples (D28).
- Display only the newest Steward label: rejected because several accepted actions can occur before a UI poll (D33).
- Re-baseline only after deferred world_reset: rejected because same-frame reset then advance loses new actions.
- Compare each settlement separately with the same global fuel: rejected because shared stock would be counted repeatedly (D32).
- Force every test world OFF or use owner preferences: rejected because either hides the production default or contaminates fixtures (D31).

## Consequences

UI observers have their own meaningful regression coverage, including failed controls, real destinations, save/reset boundaries and live translation. Functional success does not establish visual readability, physical trackpad delivery, normal-gameplay frame rate or Steward survival. Original visual failures and held-out evaluation failures remain in the Stage A report. The bounded Chronicle read size limits per-poll work, not silent loss of pending notifications.

## Date

2026-10-25.

## Source

- docs/v8/mux/MUX_GOAL.md A1–A11.
- docs/v8/mux/MUX_DECISIONS.md D11, D14, D22, D23, D28, D31–D33.
- scripts/core/sim_reads.gd; scripts/ui/mux/; scripts/ui/hud/order_bar.gd.
- tools/v8/mux_ui_test.gd; tools/v8/mux_learning_test.gd; tools/v8/mux_alerts_test.gd.

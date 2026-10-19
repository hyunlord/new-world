# Alternating switch reset investigation
Runtime: Godot4.6 GDScript; no native debugger attachment required. Existing session51711 failed at first block with switch-state mismatch; native/source identity remained unchanged.
Hypotheses: H1 deferred world_reset recreates CampSeats after the control toggle; H2 returned state dictionary aliases mutable object; H3 source/build changed during run.
Evidence: getter constructs a new dictionary; launch before==after (to verify); renderer reset replaces CampSeats with default-enabled instance. Runtime toggle proof pending.
Artifacts: external repro.gd and logs retained as required failed/successful evidence by GOAL0.9. Proposed fix limited to resetting measurement setup before first control toggle; production renderer/seat source unchanged.
References read: systematic-debugging SKILL, omo debugging SKILL, methodology00/02/06. User authorized commit/push supersedes skill permission handoff; evidence retention supersedes cleanup deletion.

Confirmed: three-case runtime repro (without wait / two-frame wait / without wait) observes false→true / false→false / false→true. H1 confirmed. Getter creates fresh Dictionary, so H2 refuted; launch before==after verified, H3 refuted. Harness-only fix adds two awaited process frames before runtime.begin and first switch. Godot parse exit0; all47 P10 Python tests pass. Full corrected session still pending.

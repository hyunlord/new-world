# Same-tick death recovery in the actual tuning cohort

Producer: `50a73474e06d3a9abbb535729e9c7e1e134954b7`; unchanged seeds 17100–17115 and original routine parameters.

Across all 16 candidate trace files, recovered pre-lethal rows are `{'ended': 146, 'consuming_interval': 114, 'started': 4, 'arrived': 4, 'end_observation': 13}`. Exactly 11 completed-on-death episodes retain their explicit completion instead of being classified as disappearance. These are actual cohort observations, in addition to the synthetic regression proof.

Hand example: seed 17101, person 27, SitByFire, chosen tick 55069, arrived tick 55088, deadline and death tick 55200. The consuming intervals sum to **55200 − 55088 = 112 minutes**, all elder minutes, with no death-tick minute added. The original observer reports `completed: false`; the corrected observer reports `completed: true`, from the actual completion event at the deadline. Both report exactly 112 consuming minutes. This episode contributes no non-elder-adult completion or minutes.

Original end row:

```json
{"adult_consuming_minutes": 0, "arrived_tick": 55088, "chosen_tick": 55069, "completed": false, "consuming_minutes": 112, "event": "ended", "id": 27, "kind": "SitByFire", "source": "agent_disappeared", "tick": 55200, "until_tick": 55200}
```

Corrected end row:

```json
{"adult_consuming_minutes": 0, "arrived_tick": 55088, "chosen_tick": 55069, "completed": true, "consuming_minutes": 112, "current_state": "Idle", "current_state_tag": 0, "current_target_kind": null, "event": "ended", "id": 27, "kind": "SitByFire", "observation_boundary": "pre_lethal", "source": "observed", "tick": 55200, "until_tick": 55200}
```

Raw roots are `/Users/rexxa/work/p17-data/a-tuning/r1/game_v21` and `/Users/rexxa/work/p17-data/a-tuning/r1-observerfix/game_v21`; original files remain unchanged.

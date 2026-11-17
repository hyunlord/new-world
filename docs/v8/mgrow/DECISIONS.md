# M-GROW decisions

Authority: owner #686, the external `goal_worldsim_mgrow.md`, and
`/Users/rexxa/work/mgrow-data/D-MGROW-1.md`. Original specifications and failed
verification evidence remain unchanged. [ADR0069](../../decisions/0069-bounded-village-growth.md)
records the M-GROW-a architecture.

| Decision | Rationale and constraint |
| --- | --- |
| Separate publishes: a paths/zones/yards, b state looks/traces/reputation/debt | Preserve the goal's order. Format24 belongs to a; format25 to b. |
| Layout data references existing zone ids | One registered identity per kind; categories choose behavior. Selected data-only zone and yard extensions must run. |
| Population caps and authored adjacency | Prevent unbounded sprawl; reject impassable/water, outcrops, conflicting fields/pens and structures. First kind may bootstrap at the hearth. |
| Housing9×9, slot4×4, people-per-zone8 | Original5×5 geometry fit only one dwelling, inconsistent with eight-person cap. Four two-bed footprints and internal spacing fit9×9. No acceptance-result tuning. |
| Steward provenance plus site-zone link | Keep existing command history and cancellation semantics while exposing the layout's reason. |
| Integer path intervals3/2 | Owner approved the existing clock's granularity. Costs and stage thresholds remain data; paved roads require construction. |
| Storage planning uses one physical envelope | Initial piles, expansion and free-slot checks reserve the future wall/door margin; usable stock cells count as Storehouse service space. Fit requests to authored interiors rather than changing zone sizes or reserve targets. |
| Safe era upgrades | Threshold750‰, full stocked materials, one active dwelling per settlement and finished reachable replacement beds before occupied-house work. |
| Read-only yard decoration | Existing GLBs only; skip conflicting positions and preload selected models. Decoration adds no inventory or terrain state. |
| Appended growth save tail | Preserve nested format23 wire order, explicitly admit authentic predecessor identities/systems, and validate all added state. |
| Baseline identity scope | Non-colony worlds stay byte-identical. Colony outcomes change through real paths/zones; the paired crash detector is the collapse floor. |
| M-GROW-b boundaries | Refuse waves plus departure-warning card; actual departures stay Inert for M8. Season-only weather-lore forecast is approved; prospect goes to M-NATION. No new hauling/traction mechanic. |

Publication requires the exact frozen commit's green pre-gate, full gate and
separate crash detector, with PLAY_ROOT/PLAY_DATA under mgrow-data and
READ_GATE_CONFIG=feature_first. No full-gate step reuse is assumed until the
M-POLISH publication and its receipt protocol are merged. Keep other passes'
measurement windows and the machine GPU lock exclusive. Record actual gate
receipts and remote SHAs externally; do not edit a gated commit to claim its
own future publication.

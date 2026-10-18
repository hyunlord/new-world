# Running frame diagnosis

Status: **NOT_VERIFIED (contended)**. W200 running2x section timers and node ablation each exhausted three separate admission holds; none launched Godot. This is an attempted diagnostic with unavailable measurements, not a renderer failure or a measured cost ranking.

| Requested quantity | Before | After |
|---|---|---|
| Every running cost >=1ms, ranked | NOT_MEASURED | NOT_MEASURED (after: contended) |
| AgentRenderer exclusive sections | NOT_MEASURED | NOT_MEASURED (after: contended) |
| Costs outside AgentRenderer, including UI panels | NOT_MEASURED | NOT_MEASURED (after: contended) |
| SimReads calls and ms per getter/frame | NOT_MEASURED | NOT_MEASURED (after: contended) |
| Tick count/frame and population series | NOT_MEASURED | NOT_MEASURED (after: contended) |

No source-only estimate is substituted for these numbers. Collector inspection confirms fresh world reset per matrix/ablation block, per-frame SimReads counters, every top-level render node and UI child (except simulation/camera/cache/container), all-on neighbours and restored controls. Corrected tooling excludes population-loss blocks from pooled metrics while retaining raw rows. These are code facts only.

See baseline.md and admission-index.json for all rejection evidence. The actual node-ablation workload is NOT_RUN due to failed admission; the diagnostic acceptance status is NOT_VERIFIED (contended), not the hour2.5 NOT_RUN fallback. Part0 finishes before its deadline and Part1 proceeds with the mandatory kernel attempt. No >=1ms claim can be made without valid runtime samples.

Part4 repeated W200 running and W500 frozen section admission three times each: all rejected before Godot. Before/after cost ranking remains unavailable; see after.md and after-admission-index.json. No source estimates substitute for frames.

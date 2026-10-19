# Declump kernel decision

**ACCEPTED** under GOAL §5.1. Six exact-clock framings have zero differing RGBA pixels and maximum channel difference zero. W0/W200/W-multi 600-frame running dumps and both W500 240-frame pans are byte-identical. Explicit off/off/on pixel stamps and off/on every-frame running stamps passed candidate validation.

| World | Control median ms | Kernel median ms | Pooled ratio | Faster pairs | Result |
|---|---:|---:|---:|---:|---|
| W500 frozen | 27.8490 | 22.1760 | 0.7962943016 | 3/3 | PASS |
| W200 running 2x | 25.7140 | 24.7935 | 0.9642023799 | 3/3 | PASS |

Each world has three 150-frame control/candidate pairs, 450 counted frames per mode. Both canonical calibrated admissions passed; raw summaries and sidecars are adjacent. W500 pair ratios: 0.7820915220, 0.7962567983, 0.7986974329. W200: 0.9582264068, 0.9657586634, 0.9729393708. The last pair need only be faster; the <=0.97 threshold applies to the pooled median and is unchanged.

Set the default on; explicit P12_DECLUMP_KERNEL=0 retains the GDScript reference. This decision accepts only the kernel on runtime baseline6d493fa8, not the simultaneously prepared write-skip candidate. The accepted-build identity reference must now be recaptured. H1-H10 final acceptance remains pending. Raw identities, frame samples and source hashes are indexed in evidence-manifest.json.

Independent read-only audit recomputed the raw ratios, decoded all RGBA controls/candidates, and checked all 2,280 running/pan frame stamps and bytes. No §5.1 acceptance gaps found. Default-on working code also passed the actual-renderer write-skip boundary suite (unit-kernel-default-on.log, exit0); write skip itself remains unaccepted and is not part of this kernel-only commit.

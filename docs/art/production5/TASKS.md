# Art Pass 5 execution ledger

Start: 2026-10-11T04:49:47Z. Six-hour boundary: 2026-10-11T10:49:47Z.
ART5_BASE_SHA: c6d88d625a86dfdc06cebe48923ae65ff1455cfb.
Host: Darwin 27.0.0 arm64, Mac only.
GOAL SHA256: f4d5f44f66c00f0f1d3125a3bb2fec4a4b1e39bfea9d7d53de638d3434eafbc6.

| Part | Start UTC | End UTC | Status / evidence |
|---|---|---|---|
| 0 baseline | 2026-10-11T04:49:47Z | 2026-10-11T05:02:14Z | PASS full gate; live snapshot keys verified |
| 1 harvest | 2026-10-11T05:01:30Z | 2026-10-11T05:10:37Z | 10 groups,38 villager frames,13 W1 selected; A20 PASS;264 Python tests PASS |
| 2 variety | 2026-10-11T05:01:30Z | 2026-10-11T05:11:21Z | exact ramps;52 Python+3 Godot tests PASS;pixel capture pending |
| 3 masks | 2026-10-11T05:01:30Z | 2026-10-11T05:21:47Z | 60/105 frame approvals;8 wood frames+15 installed masks;14 status restorations;A20 PASS |
| 4 key art | 2026-10-11T05:21:00Z | 2026-10-11T05:31:00Z |2 calls;key checkpoint STOP8/16;visual12/16+10/16;0 eligible new groups;274 Python PASS;A20 hardened/reviewed PASS |
| 5 spacing | 2026-10-11T05:31:00Z | 2026-10-11T05:31:27Z |4 headless tests PASS;seat metadata regression/re-review PASS;windowed pending |
| 6 cold | 2026-10-11T05:31:27Z | 2026-10-11T05:31:43Z |7 Godot+12 Python tests PASS;strict PASS;controlledcapture prepared;natural evidence pending |
| 7 captures | 2026-10-11T05:31:43Z | 2026-10-11T05:55:58Z | 94 art + 13 long + calibration; controlled cold scene; pre-capture full gate37/37 |
| 8 measurement | 2026-10-11T05:48:00Z | 2026-10-11T05:57:29Z | A11/A18 FAIL retained; remaining checks PASS, A19b N/A;8 artifacts;2 visual reviewers NEEDS WORK;275 Python PASS |
| 9 final | 2026-10-11T05:57:29Z | 2026-10-11T06:13:46Z (artifact seal) | gate37/37;local compatibility34/34+17static+10windows PASS;PRdraft updated; exact-HEAD publication receipt follows |

Execution follows GOAL Parts and their dependency order. Independent code preparation may run in parallel; git mutations, atlas installation, gates and captures remain serialized. All independent visual reviews use fresh sessions without pipeline/history context.

Timing convention: Part0–6 end times are verified Git commit timestamps in UTC; independent preparation began earlier in parallel. Parts5–6 formal integration intervals are shown, with prior preparation and tests recorded in their evidence folders. Earlier rounded ledger estimates were corrected against Git timestamps before publication.

Part9 artifact-seal timestamp is shown above. Post-seal exact-HEAD full-gate and remote publication finish time are recorded in `/Users/rexxa/work/art-production5-evidence/publication.json` (`finished_utc`), avoiding an endless self-referential report commit.

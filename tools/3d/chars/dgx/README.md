# M3D-2a DGX environment

Run only in `~/worldsim-3d` on `hyunlord@100.70.109.50`. Copy these scripts there.
`bash setup.sh` prepares the NGC aarch64 environment; it does not run inference.
Use `bash generate-safe.sh COMMAND ...` for every GPU item, including samples.
It holds a workspace lock, admits only GPU temperature <70°C and 1-minute load
<8, checks temperature every 2 seconds, stops only the exact owned container at
80°C, and waits 30 seconds between items. Preserve stdout/stderr in `logs/`.
Do not start another GPU item outside this wrapper. Never signal owner processes.

The container uses a host-owned workspace virtualenv, weights, caches and scratch.
Docker's image store is managed by Docker; no host system packages are installed.
The upstream TRELLIS setup script is deliberately not executed: it invokes sudo
and writes `/tmp/extensions`. Its dependency commands are reproduced inside the
workspace instead. CPU extension builds are limited to two jobs.

Upstream sources checked 2026-11-14:

- [TRELLIS.2](https://github.com/microsoft/TRELLIS.2), MIT,
  `75fbf0183001ed9876c8dbb35de6b68552ee08bd`.
- [UniRig](https://github.com/VAST-AI-Research/UniRig), MIT,
  `6793c6640ff01c8fb389f3993434124bb43d2933`.
- [NVIDIA PyTorch 25.11](https://docs.nvidia.com/deeplearning/frameworks/pytorch-release-notes/rel-25-11.html):
  CUDA 13.0.2, Blackwell support, aarch64 image selected by Docker manifest.

Hunyuan3D and AniGen are not used. Model downloads remain under
`cache/huggingface`; preserve upstream model cards/licences with the evidence.
The install logs and actual image digest/package freeze, rather than this plan,
are authoritative for successful installation. UniRig's pinned `bpy==4.2` and
its CUDA extension dependencies require separate aarch64 compatibility checks.

## Observed result (2026-11-14)

Installation is **partial**, not a proved working generation pipeline. Native
TRELLIS dependencies (`nvdiffrast`, CuMesh, FlexGEMM, `o_voxel`) built on GB10.
NGC's existing `flash_attn==2.7.4.post1+25.11` is retained; its generation CUDA
kernel has **not** been exercised. TRELLIS uses `transformers==4.57.3`; UniRig's
separate environment preserves its `4.51.3` requirement. OpenCV 5 failed to read
the shipped EXR; pinned 4.11.0.86 corrected that observed failure.

Exact upstream TRELLIS sample attempt 2 reaches gated DINOv3 and fails HTTP 401:
`facebook/dinov3-vitl16-pretrain-lvd1689m`. Workspace HF authentication is absent.
Do not substitute an unofficial mirror or an incompatible DINO architecture.
The default `briaai/RMBG-2.0` is also gated and has separate noncommercial terms;
TRELLIS's MIT code licence does not override dependency model licences.
No subject generation attempts have been consumed and no raw GLBs exist.

UniRig's `bpy==4.2` has no matching aarch64 distribution. Its exact giraffe
sample emits `ModuleNotFoundError: bpy`; its shell script prints `done` and may
exit zero despite failure. A real nonempty output is required for success.
The brief's Blender template/automatic-weights fallback remains applicable.

The tested NGC image is arm64, image ID
`sha256:eea3fe49e8a583008e4ea66fbeb36b2a7a111db8e1e322d2f310fd697baad95b`;
setup and generation pin its manifest digest. Final package freezes and failed
logs are retained externally in `m3d2-art/dgx/` and on DGX under `logs/`.


## Owner-approved resume (2026-11-15, current procedure)

The initial authentication block above is historical. The owner explicitly
approved one copy of the existing DGX token into `/workspace/.hf_token` (mode
0600); that copy is complete. Never print/copy that file into logs or Git.
Scripts read it inside the container into process-local `HF_TOKEN` only.

Do **not** run the original `example.py` unchanged: it constructs RMBG-2.0 even
for RGBA inputs. `prepare_pipeline.py` instead pins the official pipeline config
revision, writes a local config with `rembg_model=null`, and applies one exact
conditional-construction patch to the upstream pipeline. `prepare_rgba.py` uses
MIT `ZhengPeng7/BiRefNet` revision
`e2bf8e4460fc8fa32bba5ea4d94b3233d367b0e4` on CPU with two threads. It asserts
nonopaque foreground/background alpha. `infer.py` asserts RGBA/nonopaque alpha
and no constructed remover, then uses upstream alpha preprocessing. No RMBG-2.0
model is downloaded, constructed or used. UniRig is explicitly skipped on resume;
Mac Blender template skeleton/automatic weights is the approved rigging path.

The resume sample used the official `assets/example_image/T.png`, converted to
RGBA with BiRefNet and passed through the same 512 pipeline/export as the items.
It exported `raw/sample-resume-attempt1.glb`: pipeline 214.856s, guarded total
293s including initial preprocessing/downloads and 30s cooldown, peak observed
62°C. This proves sampling/native CUDA/export, which the initial blocked run
could not exercise. Actual later item statistics live in external `items.json`.

On the existing workspace, prepare the config once, then run one item at a time:

```bash
cd ~/worldsim-3d
bash generate-safe.sh python /workspace/prepare_pipeline.py > logs/resume/prepare.log 2>&1
bash run-item.sh A man 1 > logs/resume/A/man-attempt1.log 2>&1
```

Use sets `A` and `B`, subjects `man woman child goat aurochs`, attempts `1..3`.
Set A inputs are `concepts/<subject>.png`; B inputs are
`concepts/B/<subject>.png`. Each item creates `rgba/<set>/<subject>.png` if absent,
then retains its raw `raw/<set>/<subject>-attempt<N>.glb`. Existing raw output is
never overwritten. After any failure, diagnose before consuming another attempt.
`generate-safe.sh` owns exclusive GPU admission, temperature monitoring and 30s
cooldown; never overlap it or bypass it for another GPU workload.

After copying logs into the external evidence directory, extract metadata:

```bash
python3 tools/3d/chars/dgx/summarize.py /Users/rexxa/work/3d-data/m3d2-art/resume/dgx > /Users/rexxa/work/3d-data/m3d2-art/resume/dgx/items.json
```

`pipeline_elapsed_seconds` covers pipeline loading/inference/export;
`guarded_elapsed_seconds_including_30s_cooldown` additionally includes any
preprocessing in the item command. Temperature is the maximum sampled value,
not a claim about unobserved instantaneous peaks.


## Completed A/B generation

All ten subjects exported on attempt 1, no retries or safety pauses. The same
512 pipeline, seed 0, 100,000-triangle export target and 1024 texture setting
were used for both sets. Raw outputs and RGBA inputs are retained externally.
Cleanup/rig/animation and visual acceptance are separate Mac Blender checks.

| Item | Pipeline seconds | Guarded seconds incl. preprocessing + cooldown | Peak sampled °C |
|---|---:|---:|---:|
| A/aurochs | 154.882 | 221 | 48 |
| A/child | 142.196 | 210 | 48 |
| A/goat | 148.660 | 214 | 48 |
| A/man | 193.126 | 261 | 54 |
| A/woman | 145.527 | 212 | 49 |
| B/aurochs | 147.181 | 214 | 49 |
| B/child | 145.439 | 212 | 47 |
| B/goat | 159.011 | 225 | 52 |
| B/man | 150.675 | 216 | 48 |
| B/woman | 145.859 | 212 | 48 |

Ten item pipeline times total 1532.556s; guarded commands total 2197s. Subject
peak observed temperature was 54°C; the separate initial sample peaked at62°C.
Final evidence confirms zero owned running containers, GPU34°C and one-minute
load0.458. DINO's actual cached revision is
`ea8dc2863c51be0a264bab82070e3e8836b02d51`. All ten retained GLBs passed binary
header/length, triangle indices, nonempty materials/images and SHA256 evidence
checks (`raw-validation.json`). Generation success does not certify visual
quality or deformation; consult the final Korean REPORT and Blender sheets.

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

After owner-approved model access is configured **inside this workspace**, and
a permitted background-removal model is selected, resume with:

```bash
cd ~/worldsim-3d
bash generate-safe.sh bash -c 'cd /workspace/TRELLIS.2 && python example.py && test -s sample.glb' > logs/trellis-upstream-sample-resume.log 2>&1
# Only after the sample really succeeds, run one item at a time:
bash generate-safe.sh python /workspace/infer.py /workspace/concepts/man.png /workspace/raw/man-attempt1.glb > logs/man-attempt1.log 2>&1
```

The other subjects are `woman`, `child`, `goat`, `aurochs`. Keep every attempt's
raw output/log; at most three per subject. The upstream giraffe invocation is:

```bash
bash generate-safe.sh bash -c 'cd /workspace/UniRig && export PATH=/workspace/unirig-venv/bin:/usr/local/bin:/usr/bin:/bin && bash launch/inference/generate_skeleton.sh --input examples/giraffe.glb --output /workspace/raw/unirig-giraffe-skeleton.fbx && test -s /workspace/raw/unirig-giraffe-skeleton.fbx' > logs/unirig-upstream-sample-resume.log 2>&1
```

The tested NGC image is arm64, image ID
`sha256:eea3fe49e8a583008e4ea66fbeb36b2a7a111db8e1e322d2f310fd697baad95b`;
setup and generation pin its manifest digest. Final package freezes and failed
logs are retained externally in `m3d2-art/dgx/` and on DGX under `logs/`.

# WM-1 decisions

Autonomous decisions of the WM-1 run (GOAL.md 0.3.1), newest last. Each: what, why.

## Part 0

- **D0.1 Shared ssh config.** The Mac reaches the DGX with a throwaway `ssh -F` config in the session scratchpad (`Host dgx` → `hyunlord@100.70.109.50`, ControlMaster); `~/.ssh/config` untouched.
- **D0.2 Both branches created at `ce7acb25`.** `codex/worldmodel-2026-10` was pushed at `ce7acb25` (unchanged base) so it can later move by fast-forward only.
- **D0.3 Python environment: venv + PyTorch cu130, but CUDA unusable → Grace CPU.** The system interpreter (Python 3.12.3) has no torch. The venv `/home/hyunlord/work/wm1-venv` installed `torch 2.14.1+cu130` and `numpy 2.5.2` from the official cu130 index (wheel found, so cu129/cu128 were not tried). Creating a CUDA context fails at the driver level: `cuDevicePrimaryCtxRetain` and `cuCtxCreate` return 2 (`CUDA_ERROR_OUT_OF_MEMORY`) through plain `ctypes` against `libcuda.so.1`, with 7–16 GB `MemFree`, ~95 GB `MemAvailable`, no cgroup memory limit, no ollama model loaded. The only visible GPU user is another user service (`ht-estate` embed server), which this run must not stop; dropping the page cache needs root (no sudo). A docker NVIDIA container (1.3 step 3) uses the same driver and would fail the same way, so the run falls back to 1.3 step 4: **training on the Grace CPU** with this venv's torch (`torch.set_num_threads(16)`), and every run's `config.json` records the device. CUDA is re-checked before Part 2 training; if it works then, the device of every run is still recorded and the W7 rerun uses the main run's device.
- **D0.4 Dataset verification passes.** 60/60 files of one episode per split OK under `sha256sum -c`; 4,660 episode files, 1,434,758,588 bytes (1,368.3 MiB). The 192 files under `v0/schedules/` are B_act/C_cf schedule inputs, not manifest entries.

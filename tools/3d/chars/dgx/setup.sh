#!/usr/bin/env bash
# Reproduce M3D-2a dependencies inside the dedicated DGX workspace.
set -euo pipefail
cd "$HOME/worldsim-3d"
mkdir -p cache/huggingface cache/pip cache/torch cache/cuda tmp logs
if [ ! -d TRELLIS.2 ]; then git clone --recursive https://github.com/microsoft/TRELLIS.2.git; fi
if [ ! -d UniRig ]; then git clone https://github.com/VAST-AI-Research/UniRig.git; fi
git -C TRELLIS.2 checkout 75fbf0183001ed9876c8dbb35de6b68552ee08bd
git -C TRELLIS.2 submodule update --init --recursive
git -C UniRig checkout 6793c6640ff01c8fb389f3993434124bb43d2933
docker pull nvcr.io/nvidia/pytorch:25.11-py3@sha256:417cbf33f87b5378849df37983552cd1f8bc8b62fe1ceabe004de816a55dff21
# All writable paths, including the virtualenv and compiler scratch, are mounted here.
docker run --rm --read-only --name worldsim-m3d2-install --user "$(id -u):$(id -g)" \
  -v "$PWD:/workspace" -v "$PWD/tmp:/tmp" -w /workspace -e HOME=/workspace \
  -e HF_HOME=/workspace/cache/huggingface -e PIP_CACHE_DIR=/workspace/cache/pip \
  -e TORCH_HOME=/workspace/cache/torch -e CUDA_CACHE_PATH=/workspace/cache/cuda \
  -e TMPDIR=/workspace/tmp -e MAX_JOBS=2 -e CMAKE_BUILD_PARALLEL_LEVEL=2 \
  -e TORCH_CUDA_ARCH_LIST=12.1 nvcr.io/nvidia/pytorch:25.11-py3@sha256:417cbf33f87b5378849df37983552cd1f8bc8b62fe1ceabe004de816a55dff21 \
  bash /workspace/install-container.sh

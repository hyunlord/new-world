#!/usr/bin/env bash
# Install upstream dependencies without upstream setup.sh's sudo or /tmp writes.
set -euo pipefail
cd /workspace
python -m venv --system-site-packages venv
source venv/bin/activate
python -m pip install imageio imageio-ffmpeg tqdm easydict 'opencv-python-headless==4.11.0.86' ninja trimesh 'transformers==4.57.3' 'gradio==6.0.1' tensorboard pandas lpips zstandard kornia timm
# Headless inference uses utils3d.torch; omit its optional OpenGL viewer dependencies.
python -m pip install --no-deps 'git+https://github.com/EasternJournalist/utils3d.git@9a4eb15e4021b67b12c460c7057d642626897ec8'
python -m pip install plyfile
for spec in 'NVlabs/nvdiffrast nvdiffrast 253ac4fcea7de5f396371124af597e6cc957bfae' 'JeffreyXiang/CuMesh CuMesh 12289e1062f0603f2f0d0771b02e1395d247f26f' 'JeffreyXiang/FlexGEMM FlexGEMM 6dd94a859c26ee8246888502eada3dd8ad85532e'; do
  read -r repo directory revision <<< "$spec"
  if [ ! -d "$directory" ]; then git clone --recursive "https://github.com/$repo.git" "$directory"; fi
  git -C "$directory" checkout "$revision"
  git -C "$directory" submodule update --init --recursive
  python -m pip install "./$directory" --no-build-isolation
done
python -m pip install ./TRELLIS.2/o-voxel --no-build-isolation --no-deps
python -m pip install flash_attn --no-build-isolation
python -m pip freeze > logs/trellis-pip-freeze.txt
python -m venv --system-site-packages unirig-venv
source unirig-venv/bin/activate
python -m pip install transformers==4.51.3 python-box einops omegaconf pytorch_lightning lightning addict fast-simplification pyrender huggingface_hub wandb
# UniRig's bpy==4.2/open3d/flash_attn/spconv wheels may require aarch64 builds.
python -m pip install bpy==4.2 open3d spconv torch_scatter torch_cluster --no-build-isolation
python -m pip freeze > logs/pip-freeze.txt
git -C TRELLIS.2 rev-parse HEAD > logs/trellis-commit.txt
git -C UniRig rev-parse HEAD > logs/unirig-commit.txt

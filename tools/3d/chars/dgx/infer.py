#!/usr/bin/env python3
"""Export one TRELLIS.2 image as a raw textured GLB; run through generate-safe.sh."""
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Run: bash generate-safe.sh python /workspace/infer.py concepts/man.png raw/man.glb
# Uses the upstream CUDA environment prepared by setup.sh, not an isolated uv env.
import os
import sys
import time
from pathlib import Path

os.environ["OPENCV_IO_ENABLE_OPENEXR"] = "1"
os.environ["PYTORCH_CUDA_ALLOC_CONF"] = "expandable_segments:True"

import o_voxel
from PIL import Image
from trellis2.pipelines import Trellis2ImageTo3DPipeline

if len(sys.argv) != 3:
    sys.exit("usage: infer.py INPUT_IMAGE OUTPUT_GLB")
source = Path(sys.argv[1]).resolve(strict=True)
target = Path(sys.argv[2]).resolve()
workspace = Path("/workspace")
assert source.is_relative_to(workspace) and target.is_relative_to(workspace)
assert not target.exists(), "Preserve raw output; use a new attempt filename"
target.parent.mkdir(parents=True, exist_ok=True)
started = time.monotonic()
pipeline = Trellis2ImageTo3DPipeline.from_pretrained("microsoft/TRELLIS.2-4B")
pipeline.cuda()
with Image.open(source) as image:
    mesh = pipeline.run(image, seed=0, pipeline_type="512")[0]
mesh.simplify(16777216)
glb = o_voxel.postprocess.to_glb(
    vertices=mesh.vertices,
    faces=mesh.faces,
    attr_volume=mesh.attrs,
    coords=mesh.coords,
    attr_layout=mesh.layout,
    voxel_size=mesh.voxel_size,
    aabb=[[-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]],
    decimation_target=100000,
    texture_size=1024,
    remesh=True,
    remesh_band=1,
    remesh_project=0,
    verbose=True,
)
glb.export(str(target))
assert target.stat().st_size > 20, "GLB export absent or empty"
print(f"OUTPUT={target} ELAPSED_SECONDS={time.monotonic() - started:.3f}", flush=True)

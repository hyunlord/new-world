#!/usr/bin/env python3
"""Prepare local TRELLIS config that never constructs the prohibited remover."""
import json
from pathlib import Path

from huggingface_hub import hf_hub_download

workspace = Path('/workspace')
config = json.loads(Path(hf_hub_download('microsoft/TRELLIS.2-4B', 'pipeline.json', revision='af44b45f2e35a493886929c6d786e563ec68364d')).read_text())
for key, value in config['args']['models'].items():
    if value.startswith('ckpts/'):
        config['args']['models'][key] = 'microsoft/TRELLIS.2-4B/' + value
config['args']['rembg_model'] = None
local = workspace / 'pipeline-rgba'
local.mkdir(exist_ok=True)
(local / 'pipeline.json').write_text(json.dumps(config, indent=2) + '\n')
source = workspace / 'TRELLIS.2/trellis2/pipelines/trellis2_image_to_3d.py'
original = "pipeline.rembg_model = getattr(rembg, args['rembg_model']['name'])(**args['rembg_model']['args'])"
patched = original + " if args.get('rembg_model') is not None else None"
text = source.read_text()
if patched not in text:
    assert text.count(original) == 1
    source.write_text(text.replace(original, patched))
assert json.loads((local / 'pipeline.json').read_text())['args']['rembg_model'] is None
print('RGBA_ONLY_PIPELINE_READY; no background-remover constructor configured')

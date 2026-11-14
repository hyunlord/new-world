#!/usr/bin/env python3
"""Remove image background using MIT ZhengPeng7/BiRefNet on CPU, two threads."""
import os
import sys
from pathlib import Path

os.environ['HF_TOKEN'] = Path('/workspace/.hf_token').read_text().strip()
import numpy as np
import torch
from PIL import Image
from torchvision import transforms
from transformers import AutoModelForImageSegmentation

torch.set_num_threads(2)
source = Path(sys.argv[1]).resolve(strict=True)
target = Path(sys.argv[2]).resolve()
assert source.is_relative_to('/workspace') and target.is_relative_to('/workspace')
assert not target.exists()
model = AutoModelForImageSegmentation.from_pretrained('ZhengPeng7/BiRefNet', revision='e2bf8e4460fc8fa32bba5ea4d94b3233d367b0e4', trust_remote_code=True).eval()
transform = transforms.Compose([transforms.Resize((1024, 1024)), transforms.ToTensor(), transforms.Normalize([0.485, 0.456, 0.406], [0.229, 0.224, 0.225])])
with Image.open(source) as original:
    image = original.convert('RGB')
    with torch.inference_mode():
        alpha = model(transform(image).unsqueeze(0))[-1].sigmoid().squeeze().cpu()
    mask = transforms.ToPILImage()(alpha).resize(image.size, Image.Resampling.LANCZOS)
    image.putalpha(mask)
    pixels = np.asarray(image)[:, :, 3]
    assert np.any(pixels < 128) and np.any(pixels > 204), 'Empty or opaque mask'
    target.parent.mkdir(parents=True, exist_ok=True)
    image.save(target)
print(f'RGBA_READY={target}', flush=True)

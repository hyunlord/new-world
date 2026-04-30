# Wildlife Sprite Session Handoff — A4

## Context

After A4 (wildlife-sprite-visualization-v1), the rendering pipeline is fully wired:
`WildlifeRuntimeSystem → SimBridge.get_wildlife_snapshots() → entity_renderer.gd → Sprite2D`

Placeholder PNGs are serving until this session generates real pixel art.

## What's Already Done

- `WildlifeSnapshot` 24-byte struct + `build_wildlife_snapshots()` (sim-engine)
- `SimBridge.get_wildlife_snapshots()` FFI method
- `entity_renderer.gd`: load textures, decode bytes, update sprites per tick
- Placeholder PNGs (wolf/bear/boar) in `assets/sprites/wildlife/`

## What Sprite Session Needs to Do

### Goal
Replace the 3 placeholder PNGs with pixel art via ComfyUI.
Placeholders are 32×32 RGBA — keep the same size for drop-in compatibility.

### File Targets
```
assets/sprites/wildlife/wolf.png   (32×32 RGBA)
assets/sprites/wildlife/bear.png   (32×32 RGBA)
assets/sprites/wildlife/boar.png   (32×32 RGBA)
```

### ComfyUI Configuration
Use the established stone-age pixel art pipeline:
- Base: SDXL 1.0
- LoRA: Pixel Art XL (NeriJS)
- IPAdapter Plus + CLIP-H
- Pixelization: comfy_pixelization
- Background removal: rembg

### Style Guide
- Stone-age palette (earth tones)
- Side view (matches agent_base.png convention)
- Neutral idle pose
- 32×32 strict size
- Transparent background (RGBA)

### Subjects
- **Wolf**: gray fur, lean predator, side view
- **Bear**: brown fur, large heavy form, side view
- **Boar**: pinkish-brown, tusks visible, side view

### Workflow
1. Generate 16 candidates per kind (48 total)
2. Pixelate to 32×32
3. Background remove with rembg
4. Aseprite cleanup if needed
5. Pick best 1 per kind
6. Save to `assets/sprites/wildlife/`
7. Commit: `feat(wildlife-sprites): replace placeholders with ComfyUI variants`

### Validation After Replacement
```bash
# In the game session repo (new-world, not worldsim-training):
python3 -c "
from PIL import Image
for name in ['wolf', 'bear', 'boar']:
    img = Image.open(f'assets/sprites/wildlife/{name}.png')
    assert img.size == (32, 32), f'{name}: wrong size {img.size}'
    assert img.mode == 'RGBA', f'{name}: wrong mode {img.mode}'
    print(f'{name}: OK')
"
```

## Out of Scope for Sprite Session
- Animation frames (static 1-frame is correct for A4)
- Modular layers (head/body separate)
- Combat poses or death sprites
- Sprite atlas packing

## Reference
- Existing sprites: `assets/sprites/buildings/`, `assets/sprites/walls/`
- agent_base.png is 64×72 — wildlife sprites are intentionally smaller (32×32)

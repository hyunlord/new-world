# T-450: Building Renderer LOD

## Action: DISPATCH (Codex)
## Files: scripts/ui/building_renderer.gd

### Zoom levels (camera zoom.x):
- **Strategic (zoom < 1.5)**: 2-3px color block per building type
- **Town (1.5 <= zoom < 4.0)**: current rendering (filled shapes, outlines, progress bars)
- **Detail (zoom >= 4.0)**: current + storage text labels on stockpiles (e.g. "F:12 W:45")

### Implementation:
```gdscript
func _draw() -> void:
    var zoom_level: float = 1.0
    var cam: Camera2D = get_viewport().get_camera_2d()
    if cam != null:
        zoom_level = cam.zoom.x
    # Branch on zoom_level
```

### Hysteresis: ±0.2 buffer with _current_lod_level

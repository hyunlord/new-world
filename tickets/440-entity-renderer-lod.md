# T-440: Entity Renderer LOD

## Action: DISPATCH (Codex)
## Files: scripts/ui/entity_renderer.gd

### Zoom levels (camera zoom.x):
- **Strategic (zoom < 1.5)**: 1px dot, no carrying indicator, no names
- **Town (1.5 <= zoom < 4.0)**: current rendering (job shapes, carrying dots, hunger warning)
- **Detail (zoom >= 4.0)**: current + selected entity name label

### Implementation:
```gdscript
func _draw() -> void:
    var zoom_level: float = 1.0
    var cam: Camera2D = get_viewport().get_camera_2d()
    if cam != null:
        zoom_level = cam.zoom.x
    # Then branch on zoom_level
```

### Hysteresis: use ±0.2 buffer with instance var _current_lod_level

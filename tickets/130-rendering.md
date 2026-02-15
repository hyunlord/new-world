# 130 - Rendering (WorldRenderer + EntityRenderer + CameraController)

## Objective
Create the visual layer: world map as single image, entity dots drawn with _draw(), and camera with zoom/pan.

## Prerequisites
- 020-game-config
- 070-world-data
- 090-entity-system

## Non-goals
- No animations
- No particle effects
- No lighting/shadows

## Files to create
- `scripts/ui/world_renderer.gd`
- `scripts/ui/entity_renderer.gd`
- `scripts/ui/camera_controller.gd`

## Implementation Steps

### WorldRenderer
1. Create `scripts/ui/world_renderer.gd` extending Sprite2D
2. ```gdscript
   class_name WorldRenderer
   extends Sprite2D
   ```
3. `render_world(world_data: WorldData) -> void`:
   - Create Image (world_data.width, world_data.height, false, Image.FORMAT_RGB8)
   - For each tile: set pixel color from biome color + elevation brightness
   - Color = base_biome_color * lerp(0.7, 1.3, elevation)
   - Create ImageTexture from Image
   - self.texture = texture
   - self.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
   - self.scale = Vector2(GameConfig.TILE_SIZE, GameConfig.TILE_SIZE)
   - Position at world center: Vector2(width * TILE_SIZE / 2, height * TILE_SIZE / 2)

### EntityRenderer
1. Create `scripts/ui/entity_renderer.gd` extending Node2D
2. ```gdscript
   class_name EntityRenderer
   extends Node2D

   var _entity_manager: EntityManager
   var selected_entity_id: int = -1
   ```
3. `init(entity_manager: EntityManager)`: store ref
4. `_process(_delta)`: queue_redraw()
5. `_draw()`:
   - For each alive entity:
     - pos = Vector2(entity.position) * GameConfig.TILE_SIZE + Vector2(GameConfig.TILE_SIZE/2, GameConfig.TILE_SIZE/2)
     - base_color = Color.WHITE
     - Modulate by action: rest=Color(0.5,0.5,0.7), seek_food=Color.YELLOW, socialize=Color.CYAN, wander=Color.WHITE
     - draw_circle(pos, 4.0, color)
     - If hunger < 0.2: draw_circle(pos + Vector2(0,-6), 2.0, Color.RED)
     - If entity.id == selected_entity_id:
       - draw_arc(pos, 6.0, 0, TAU, 16, Color.WHITE, 1.5)
       - If action_target valid: draw_dashed_line(pos, target_pos, Color(1,1,1,0.3))
6. `_input(event)`:
   - If InputEventMouseButton, left click:
     - Convert to world coords
     - tile = Vector2i(world_pos / GameConfig.TILE_SIZE)
     - Find entity at tile → select
     - SimulationBus.entity_selected.emit(entity_id) or entity_deselected

### CameraController
1. Create `scripts/ui/camera_controller.gd` extending Camera2D
2. ```gdscript
   class_name CameraController
   extends Camera2D

   var _target_zoom: float = 1.0
   var _is_dragging: bool = false
   var _drag_start: Vector2 = Vector2.ZERO
   ```
3. `_ready()`:
   - Set initial position to world center
   - zoom = Vector2(1, 1)
4. `_unhandled_input(event)`:
   - Mouse wheel up: _target_zoom = clamp(_target_zoom + ZOOM_STEP, MIN, MAX)
   - Mouse wheel down: _target_zoom = clamp(_target_zoom - ZOOM_STEP, MIN, MAX)
   - Middle button pressed: _is_dragging = true, _drag_start = event.position
   - Middle button released: _is_dragging = false
   - Mouse motion while dragging: position -= (event.position - _drag_start) / zoom.x; _drag_start = event.position
5. `_process(delta)`:
   - Smooth zoom: zoom = zoom.lerp(Vector2(_target_zoom, _target_zoom), ZOOM_SPEED)
   - WASD pan: direction * PAN_SPEED * delta / zoom.x
   - Clamp position to world bounds

## Verification
- Gate PASS

## Acceptance Criteria
- [ ] World renders as colored pixel map
- [ ] Entities shown as colored dots
- [ ] Selected entity highlighted
- [ ] Camera zoom/pan works
- [ ] Mouse wheel zoom centered on cursor (stretch goal, basic is fine)
- [ ] Gate PASS

## Risk Notes
- queue_redraw() every frame may be costly with many entities (acceptable for 500)
- WorldRenderer only re-renders on world change, not every frame
- draw_dashed_line needs manual implementation (Godot 4 has it)

## Roll-back Plan
- Delete files

# 150 - Main Scene (Wire Everything Together)

## Objective
Create the main scene that initializes all systems, generates the world, spawns entities, and runs the simulation loop.

## Prerequisites
- ALL previous tickets (010-140)

## Non-goals
- No save/load UI (console only)
- No menu screen

## Files to create
- `scenes/main/main.tscn`
- `scenes/main/main.gd`

## Implementation Steps
1. Create `scenes/main/main.gd` extending Node2D:
   ```gdscript
   extends Node2D

   var sim_engine: SimulationEngine
   var world_data: WorldData
   var world_generator: WorldGenerator
   var entity_manager: EntityManager
   var needs_system: NeedsSystem
   var behavior_system: BehaviorSystem
   var movement_system: MovementSystem

   @onready var world_renderer: WorldRenderer = $WorldRenderer
   @onready var entity_renderer: EntityRenderer = $EntityRenderer
   @onready var camera: CameraController = $Camera
   @onready var hud: HUD = $HUD
   ```

2. `_ready()`:
   - Print controls to console
   - Initialize seed from GameConfig.WORLD_SEED
   - Create SimulationEngine, init_with_seed(seed)
   - Create WorldData, init_world(WORLD_SIZE)
   - Create WorldGenerator, generate(world_data, seed)
   - Create EntityManager, init(world_data, sim_engine.rng)
   - Create NeedsSystem, init(entity_manager)
   - Create BehaviorSystem, init(entity_manager, world_data, sim_engine.rng)
   - Create MovementSystem, init(entity_manager, world_data)
   - Register all systems with sim_engine
   - Render world: world_renderer.render_world(world_data)
   - Init entity_renderer with entity_manager
   - Init HUD references
   - Spawn INITIAL_SPAWN_COUNT entities on walkable tiles near center
   - Position camera at world center

3. `_spawn_initial_entities()`:
   - Find walkable tiles within 30 tiles of center
   - Randomly pick INITIAL_SPAWN_COUNT positions
   - spawn_entity for each

4. `_process(delta)`:
   - sim_engine.update(delta)
   - Update HUD entity/event counts

5. `_unhandled_input(event)`:
   - If action "pause_toggle" pressed: sim_engine.toggle_pause()
   - If action "speed_up": sim_engine.increase_speed()
   - If action "speed_down": sim_engine.decrease_speed()

6. Create `scenes/main/main.tscn`:
   - Root: Node2D (main.gd attached)
   - Children: WorldRenderer (Sprite2D), EntityRenderer (Node2D), Camera (Camera2D), HUD (CanvasLayer)

7. Set as main scene in project.godot: `run/main_scene="res://scenes/main/main.tscn"`

## Verification
- Gate PASS
- F5 runs and shows world + entities
- All controls work

## Acceptance Criteria
- [ ] F5 launches game
- [ ] 256x256 colorful island world visible
- [ ] 20 entity dots moving autonomously
- [ ] Click entity shows info
- [ ] Space pauses/resumes
- [ ] ,/. changes speed
- [ ] WASD/mouse pan and zoom
- [ ] Console shows events
- [ ] Same seed = same world
- [ ] Gate PASS

## Risk Notes
- Node order matters: WorldRenderer must be behind EntityRenderer
- Camera must be current
- HUD on CanvasLayer so it's not affected by camera

## Roll-back Plan
- Delete files, unset main_scene

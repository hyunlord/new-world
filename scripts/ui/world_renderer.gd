extends Node2D

# WorldRenderer — Node2D child of Main scene.
# T7.9.B render mechanism: pulls Warmth influence overlay from WorldSim
# and uploads it to a Sprite2D texture every frame.
#
# Bootstrap: places one building at (32, 32) radius 8 so the BuildingStamp
# system has something to drive. Visual will remain mostly black until the
# Phase 2 propagation wiring (T7.10) lands.

const TILE_SIZE := 16
const GRID_W := 64
const GRID_H := 64
const CHANNEL_WARMTH := 0
const BOOTSTRAP_X := 32
const BOOTSTRAP_Y := 32
const BOOTSTRAP_RADIUS := 8

var world_sim: WorldSimNode
var sprite: Sprite2D
var texture: ImageTexture
var image: Image

func _ready() -> void:
	print("WorldRenderer ready (T7.9.B render mechanism)")
	world_sim = get_node("../WorldSim") as WorldSimNode
	if world_sim == null:
		push_error("WorldSim node not found at ../WorldSim")
		return
	world_sim.on_building_placed(BOOTSTRAP_X, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)
	image = Image.create(GRID_W, GRID_H, false, Image.FORMAT_L8)
	texture = ImageTexture.create_from_image(image)
	sprite = Sprite2D.new()
	sprite.texture = texture
	sprite.scale = Vector2(TILE_SIZE, TILE_SIZE)
	sprite.position = Vector2(960, 540)
	add_child(sprite)

func _process(_delta: float) -> void:
	if world_sim == null:
		return
	var data: PackedByteArray = world_sim.get_influence_overlay(CHANNEL_WARMTH)
	if data.size() != GRID_W * GRID_H:
		return
	image = Image.create_from_data(GRID_W, GRID_H, false, Image.FORMAT_L8, data)
	texture.update(image)

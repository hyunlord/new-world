extends Node2D

# WorldRenderer — Node2D child of Main scene.
# T7.9.B render mechanism: pulls an influence channel overlay from WorldSim
# and uploads it to a Sprite2D texture every frame.
# T7.10.B1: SPACE toggles between Warmth (T7.10.A) and Light (T7.10.B) channels.
# T7.10.C: cycle extended to include Noise (linear-decay) — SPACE cycles
# Warmth → Light → Noise → Warmth so all three backend wirings can be
# confirmed visually in one F6 session.
#
# Bootstrap: places one building at (32, 32) radius 8 so the BuildingStamp
# system has something to drive. Initial channel = Warmth.

const TILE_SIZE := 16
const GRID_W := 64
const GRID_H := 64
const CHANNEL_WARMTH := 0
const CHANNEL_LIGHT := 1
const CHANNEL_NOISE := 2
const BOOTSTRAP_X := 32
const BOOTSTRAP_Y := 32
const BOOTSTRAP_RADIUS := 8

var current_channel: int = CHANNEL_WARMTH
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

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_SPACE:
			# T7.10.C: 3-state cycle Warmth → Light → Noise → Warmth.
			var channel_name: String
			if current_channel == CHANNEL_WARMTH:
				current_channel = CHANNEL_LIGHT
				channel_name = "Light"
			elif current_channel == CHANNEL_LIGHT:
				current_channel = CHANNEL_NOISE
				channel_name = "Noise"
			else:
				current_channel = CHANNEL_WARMTH
				channel_name = "Warmth"
			print("Channel switched: ", channel_name)

func _process(_delta: float) -> void:
	if world_sim == null:
		return
	var data: PackedByteArray = world_sim.get_influence_overlay(current_channel)
	if data.size() != GRID_W * GRID_H:
		return
	image = Image.create_from_data(GRID_W, GRID_H, false, Image.FORMAT_L8, data)
	texture.update(image)

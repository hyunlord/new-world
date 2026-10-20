# terrain_layer.gd (ColonyView/TerrainLayer in scenes/main.tscn): the ground of the M1 colony view.
# A TileMapLayer built once per world from the terrain snapshot: grass (4 variants, darker patches from the
#   elevation byte), forest floor, hill, mountain rock, river water (shallow next to land) and packed dirt
#   inside the camp clearing. The calmest layer: low contrast, no animation.
# Bridge methods: via SimReads get_terrain_snapshot, get_building_snapshot; signal world_reset.
extends TileMapLayer

const View = preload("res://scripts/world/colony/view.gd")
const NAMES: Array[String] = [
	"grass_0", "grass_1", "grass_2", "grass_3", "grass_dark_0", "grass_dark_1", "dirt_0", "dirt_1",
	"water_0", "water_1", "shallow", "hill", "mountain", "forest_floor_0", "forest_floor_1",
]
# TerrainType codes (sim-core material/terrain.rs): 0 Plain, 1 Forest, 2 Mountain, 3 Hill, 4 River.
const PLAIN: int = 0
const FOREST: int = 1
const MOUNTAIN: int = 2
const HILL: int = 3
const RIVER: int = 4
const CLEARING: int = 6

var _world_sim: Node = null
var _coords: Dictionary = {}
var _pending: bool = true


func _ready() -> void:
	z_index = 0
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	var built: Dictionary = View.tileset(NAMES)
	tile_set = built["tileset"]
	_coords = built["coords"]
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_pending = true


func _process(_delta: float) -> void:
	if _pending and _world_sim != null:
		refresh()


## Rebuild every cell from the terrain snapshot.
func refresh() -> void:
	var snap: Dictionary = SimReads.read(_world_sim, "get_terrain_snapshot")
	var w: int = int(snap.get("width", 0))
	var h: int = int(snap.get("height", 0))
	var terrain: PackedByteArray = snap.get("terrain", PackedByteArray())
	var elevation: PackedByteArray = snap.get("elevation", PackedByteArray())
	if w <= 0 or terrain.size() != w * h:
		return
	_pending = false
	var hearth: Vector2i = Vector2i(-1000, -1000)
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var bxs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(bxs.size(), bys.size())):
		if kinds[i] == 0:
			hearth = Vector2i(bxs[i], bys[i])
			break
	# Buildings register on the first tick: until then draw the ground and retry for the clearing.
	_pending = hearth.x < 0
	clear()
	for y: int in h:
		for x: int in w:
			var code: int = terrain[y * w + x]
			var hash_value: int = View.tile_hash(x, y)
			var name: String
			match code:
				FOREST:
					name = "forest_floor_%d" % (hash_value % 2)
				MOUNTAIN:
					name = "mountain"
				HILL:
					name = "hill"
				RIVER:
					name = "shallow" if _land_next_to(terrain, w, h, x, y) else "water_%d" % (hash_value % 2)
				_:
					var d: int = maxi(absi(x - hearth.x), absi(y - hearth.y))
					if d <= CLEARING - 2 or (d <= CLEARING and hash_value % 3 == 0):
						name = "dirt_%d" % (hash_value % 2)
					elif elevation.size() == terrain.size() and elevation[y * w + x] > 150 and hash_value % 4 != 0:
						name = "grass_dark_%d" % (hash_value % 2)
					else:
						name = "grass_%d" % (hash_value % 4)
			set_cell(Vector2i(x, y), 0, _coords.get(name, Vector2i.ZERO))


func _land_next_to(terrain: PackedByteArray, w: int, h: int, x: int, y: int) -> bool:
	for d: Vector2i in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
		var nx: int = x + d.x
		var ny: int = y + d.y
		if nx >= 0 and ny >= 0 and nx < w and ny < h and terrain[ny * w + nx] != RIVER:
			return true
	return false

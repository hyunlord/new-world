# Slice 1 texture arrays and opt-in shader variants. Bridge methods called: none.
extends RefCounted

# Layer order is shared with slice_pbr.gdshaderinc.
const ASSETS: Array[String] = [
	"grass_ground", "forest_floor", "dirt_floor", "mud_forest", "rock_face",
	"jolcham_oak_bark_01", "pine_bark", "wood_trunk_wall", "wood_planks_dirt", "thatch_roof_angled",
]
const MAPS: Array[String] = ["diff", "nor_gl", "rough", "ao"]
const PARAMETERS: Array[StringName] = [&"slice_albedo", &"slice_normal", &"slice_roughness", &"slice_ao"]
const LAYER_SIZE: int = 1024
const S2_ROOT: String = "res://assets/3d/anchor2/"
const S2_SIZE: int = 256
static var _textures: Array[Texture2DArray] = []
static var _variants: Dictionary = {}


static func style2() -> bool:
	return OS.get_cmdline_user_args().has("--style=s2")


static func load_textures() -> bool:
	if not _textures.is_empty():
		return true
	var folder: String = OS.get_environment("WORLDSIM_SLICE1_ASSETS")
	if folder.is_empty():
		folder = ProjectSettings.globalize_path("res://../art-img-data/slice1/assets")
	var textures: Array[Texture2DArray] = []
	var styled: bool = style2()
	for channel: int in MAPS.size():
		var images: Array[Image] = []
		var strip: Image
		if styled:
			var source: Texture2D = load(S2_ROOT + "surface_%s.png" % MAPS[channel]) as Texture2D
			strip = source.get_image() if source != null else null
			if strip == null or strip.get_size() != Vector2i(S2_SIZE * ASSETS.size(), S2_SIZE):
				push_error("Style 2 surface strip missing or invalid: %s" % MAPS[channel])
				return false
		for index: int in ASSETS.size():
			var image: Image
			if styled:
				image = strip.get_region(Rect2i(index * S2_SIZE, 0, S2_SIZE, S2_SIZE))
			else:
				var asset: String = ASSETS[index]
				var resolution: String = "1k" if asset == "jolcham_oak_bark_01" else "2k"
				var path: String = folder.path_join(asset).path_join("%s_%s_%s.jpg" % [asset, MAPS[channel], resolution])
				image = Image.load_from_file(path)
				if image == null or image.is_empty() or maxi(image.get_width(), image.get_height()) > 2048:
					push_error("Slice PBR texture unavailable or above 2K at %s" % path)
					return false
			image.convert(Image.FORMAT_RGB8 if channel < 2 else Image.FORMAT_R8)
			if not styled:
				image.resize(LAYER_SIZE, LAYER_SIZE, Image.INTERPOLATE_LANCZOS)
			image.generate_mipmaps(channel == 1)
			images.append(image)
		var texture: Texture2DArray = Texture2DArray.new()
		if texture.create_from_images(images) != OK or texture.get_layers() != ASSETS.size():
			push_error("Slice PBR texture array creation failed for %s" % MAPS[channel])
			return false
		textures.append(texture)
	_textures = textures
	return true


## ShaderMaterial keeps its regular parameter overrides when switched between variants.
static func configure(material: ShaderMaterial, standard: Shader, enabled: bool) -> bool:
	if not enabled:
		material.shader = standard
		return true
	if not load_textures():
		return false
	if not _variants.has(standard):
		var variant: Shader = Shader.new()
		variant.code = "#define SLICE_PBR\n" + standard.code
		_variants[standard] = variant
	material.shader = _variants[standard]
	for channel: int in PARAMETERS.size():
		material.set_shader_parameter(PARAMETERS[channel], _textures[channel])
	return true


static func slot_is_pbr(slot: int) -> bool:
	return slot in [4, 5, 6, 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26]

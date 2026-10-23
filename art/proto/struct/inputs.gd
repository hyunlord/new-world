# Purpose: load and hash the exact STRUCT crowd inputs without importing game code.
# Bridge methods called: none. External image files are isolated laboratory inputs.
extends RefCounted

const STATES := ["idle", "walk_a", "work_a", "carry", "sleep"]
var hashes: Dictionary = {}
var people: Array[Dictionary] = []
var environment: Array[ImageTexture] = []
var environment_paths: Array[String] = []
var rooms: Array[Dictionary] = []
var ground: ImageTexture
var failure := ""

func load_all(root: String) -> bool:
    var route := root.path_join("art/character/route_i")
    var adopted_path := route.path_join("MANIFEST.json")
    var adopted_bytes := FileAccess.get_file_as_bytes(adopted_path)
    var adopted: Variant = JSON.parse_string(adopted_bytes.get_string_from_utf8())
    if not adopted is Dictionary or adopted.get("accepted_count", 0) != 34:
        failure = "missing adopted 34-look manifest"
        return false
    hashes[adopted_path] = _digest(adopted_bytes)
    var palette_path := root.path_join("art/compositor/top_colours.json")
    var palette_bytes := FileAccess.get_file_as_bytes(palette_path)
    var palette: Variant = JSON.parse_string(palette_bytes.get_string_from_utf8())
    if not palette is Dictionary or palette.size() != 34:
        failure = "missing measured top palette"
        return false
    hashes[palette_path] = _digest(palette_bytes)
    for entry: Dictionary in adopted["accepted_looks"]:
        var look := str(entry["look"])
        var manifest_path := route.path_join(str(entry["manifest"]))
        var manifest_bytes := FileAccess.get_file_as_bytes(manifest_path)
        if _digest(manifest_bytes) != str(entry["sha256"]):
            failure = "adopted manifest digest changed: " + look
            return false
        hashes[manifest_path] = str(entry["sha256"])
        var manifest: Dictionary = JSON.parse_string(manifest_bytes.get_string_from_utf8())
        var textures: Array[ImageTexture] = []
        for facing: String in ["S", "E"]:
            for state: String in STATES:
                var path := route.path_join(look).path_join(facing).path_join(state + ".png")
                var texture := _texture(path)
                if texture == null:
                    return false
                if str(hashes[path]) != str(manifest["files"].get(facing + "/" + state + ".png", "")):
                    failure = "adopted frame digest changed: " + path
                    return false
                textures.append(texture)
        var channels: Array = palette[look]
        var colour := Color(float(channels[0]) / 255.0, float(channels[1]) / 255.0, float(channels[2]) / 255.0)
        people.append({"look": look, "textures": textures, "colour": colour})
    var manifest_path := root.path_join("docs/v8/artstruct/assets.json")
    var manifest_bytes := FileAccess.get_file_as_bytes(manifest_path)
    var manifest: Variant = JSON.parse_string(manifest_bytes.get_string_from_utf8())
    if not manifest is Dictionary or not manifest.get("assets") is Array:
        failure = "missing STRUCT asset manifest"
        return false
    hashes[manifest_path] = _digest(manifest_bytes)
    for entry: Dictionary in manifest["assets"]:
        var asset_id := str(entry["id"])
        if asset_id.ends_with("_face") or asset_id.ends_with("_cap") or asset_id.ends_with("_end") or asset_id.contains("corner") or asset_id == "water" or asset_id.begins_with("ford_stone"):
            continue
        var path := root.path_join(str(entry["path"]))
        var texture := _texture(path)
        if texture == null:
            return false
        if texture.get_width() > 320 or texture.get_height() > 192:
            failure = "oversize standing object: " + asset_id
            return false
        environment.append(texture)
        environment_paths.append(path)
    if environment.size() < 10:
        failure = "fewer than ten actual STRUCT environment variants"
        return false
    if not _load_rooms(root):
        return false
    ground = _texture(root.path_join("art/proto/env1/ground.png"))
    return ground != null

func _load_rooms(root: String) -> bool:
    var path := root.path_join("art/proto/struct/rooms.json")
    var bytes := FileAccess.get_file_as_bytes(path)
    var manifest: Variant = JSON.parse_string(bytes.get_string_from_utf8())
    if not manifest is Dictionary or not manifest.get("rooms") is Array or manifest["rooms"].size() != 3 or not manifest.get("bake_inputs") is Dictionary or manifest["bake_inputs"].is_empty():
        failure = "missing three-room bake manifest"
        return false
    hashes[path] = _digest(bytes)
    for relative: String in manifest["bake_inputs"]:
        var input_path := root.path_join(relative)
        var expected := str(manifest["bake_inputs"][relative])
        if FileAccess.get_sha256(input_path) != expected:
            failure = "baked source changed: " + relative
            return false
        hashes[input_path] = expected
    for entry: Dictionary in manifest["rooms"]:
        var texture := _texture(root.path_join(str(entry["path"])))
        if texture == null or str(hashes[root.path_join(str(entry["path"]))]) != str(entry["sha256"]):
            failure = "room texture missing or hash mismatch"
            return false
        var position: Array = entry["position"]
        rooms.append({"texture": texture, "position": Vector2(float(position[0]), float(position[1]))})
    return true

func _texture(path: String) -> ImageTexture:
    var bytes := FileAccess.get_file_as_bytes(path)
    var image := Image.new()
    if image.load_png_from_buffer(bytes) != OK or image.is_empty():
        failure = "missing input: " + path
        return null
    hashes[path] = _digest(bytes)
    return ImageTexture.create_from_image(image)

func _digest(bytes: PackedByteArray) -> String:
    var context := HashingContext.new()
    if context.start(HashingContext.HASH_SHA256) != OK or context.update(bytes) != OK:
        failure = "input hashing failed"
        return ""
    return context.finish().hex_encode()

func unchanged() -> bool:
    for path: String in hashes:
        if FileAccess.get_sha256(path) != str(hashes[path]):
            return false
    return true

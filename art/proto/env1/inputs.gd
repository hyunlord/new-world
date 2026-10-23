# Purpose: load and hash the exact ENV1 crowd inputs without importing game code.
# Bridge methods called: none. External image files are isolated laboratory inputs.
extends RefCounted

const STATES := ["idle", "walk_a", "work_a", "carry", "sleep"]
var hashes: Dictionary = {}
var people: Array[Dictionary] = []
var environment: Array[ImageTexture] = []
var environment_paths: Array[String] = []
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
    for filename: String in ["built_assets.json", "items_manifest.json"]:
        var manifest_path := root.path_join("docs/v8/artenv1").path_join(filename)
        var manifest_bytes := FileAccess.get_file_as_bytes(manifest_path)
        var manifest: Variant = JSON.parse_string(manifest_bytes.get_string_from_utf8())
        if not manifest is Dictionary or not manifest.get("assets") is Array:
            failure = "missing environment manifest: " + filename
            return false
        hashes[manifest_path] = _digest(manifest_bytes)
        for entry: Dictionary in manifest["assets"]:
            if str(entry["path"]).contains("snow"):
                continue
            if not _environment_asset(root.path_join(str(entry["path"])), str(entry["sha256"])):
                return false
        if filename == "items_manifest.json":
            var spoiled: Dictionary = manifest["spoiled_meat"]
            if not _environment_asset(root.path_join(str(spoiled["path"])), str(spoiled["sha256"])):
                return false
    if environment.size() < 10:
        failure = "fewer than ten actual ENV1 environment variants"
        return false
    ground = _texture(root.path_join("art/proto/env1/ground.png"))
    return ground != null

func _environment_asset(path: String, expected_hash: String) -> bool:
    var texture := _texture(path)
    if texture == null:
        return false
    if str(hashes[path]) != expected_hash:
        failure = "environment digest changed: " + path
        return false
    if texture.get_width() <= 192 and texture.get_height() <= 192:
        environment.append(texture)
        environment_paths.append(path)
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

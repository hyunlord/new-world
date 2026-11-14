# renderer_switch.gd — which view draws the world: the 2D colony view (default) or the 3D view (M3D-1).
# Purpose: `--renderer=3d` / `--renderer=2d` after `--` on the command line wins; otherwise the interface setting
#   `renderer_3d` (settings.cfg, settings_store.gd) decides. The 2D view stays the default until the owner approves
#   3D parity (GOAL_3D rule 3).
# Bridge methods called: none.
extends RefCounted

const Store = preload("res://scripts/ui/mux/settings_store.gd")


## True when this run should draw the world in 3D.
static func wants_3d() -> bool:
	for arg: String in OS.get_cmdline_user_args() + OS.get_cmdline_args():
		if arg == "--renderer=3d":
			return true
		if arg == "--renderer=2d":
			return false
	var path: String = Store.resolve_path()
	if path.is_empty():
		return false
	var config: ConfigFile = ConfigFile.new()
	if config.load(path) != OK:
		return false
	return bool(config.get_value("interface", "renderer_3d", false))

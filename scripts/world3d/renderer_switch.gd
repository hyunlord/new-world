# renderer_switch.gd — which view draws the world: the 3D view (the default since M3D-SWITCH E1) or the 2D colony view.
# Purpose: `--renderer=2d` (or `--renderer=3d`) on the command line wins; nothing else picks 2D. The 2D view is an
#   escape hatch only until M3D-SWITCH E2 deletes it; the former `renderer_3d` interface setting is no longer read.
# Bridge methods called: none.
extends RefCounted

## True when this run should draw the world in 3D.
static func wants_3d() -> bool:
	for arg: String in OS.get_cmdline_user_args() + OS.get_cmdline_args():
		if arg == "--renderer=3d":
			return true
		if arg == "--renderer=2d":
			return false
	return true

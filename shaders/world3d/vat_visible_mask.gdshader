// Marks pixels where an animated body passes the opaque scene depth test.
shader_type spatial;
render_mode unshaded, depth_draw_never, cull_back, shadows_disabled, fog_disabled;
stencil_mode write, 27;

#include "res://shaders/world3d/vat_anim.gdshaderinc"

void vertex() {
	float dist = length((MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz - CAMERA_POSITION_WORLD);
	VERTEX = vat_position(UV2, INSTANCE_CUSTOM, dist) * far_scale_at(dist);
}

void fragment() {
	ALPHA = 0.0;
}

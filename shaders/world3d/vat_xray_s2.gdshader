// vat_xray_s2.gdshader — Style 2 see-through pass (scripts/world3d/vat3d.gd; the legacy style keeps vat_xray.gdshader).
// Draw hidden fragments after the visible-body stencil pass, with one blend per pixel.
// Visible body pixels and any silhouette pixel already drawn have stencil value 27.
shader_type spatial;
render_mode unshaded, depth_test_inverted, depth_draw_never, blend_mix, cull_back, shadows_disabled, fog_disabled;
stencil_mode read, write, compare_not_equal, 27;

#include "res://shaders/world3d/vat_anim.gdshaderinc"
global uniform float w3d_night;
uniform vec4 day_colour : source_color = vec4(0.06, 0.055, 0.05, 0.9);
uniform vec4 night_colour : source_color = vec4(0.57, 0.65, 0.66, 0.54);

uniform bool distant_only = false;
varying float outline_weight;
varying float wardrobe_value;

void vertex() {
	float dist = length((MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz - CAMERA_POSITION_WORLD);
	outline_weight = smoothstep(34.0, 44.0, dist);
	wardrobe_value = mix(1.0, mix(0.64, 1.26, COLOR.a), smoothstep(78.0, 102.0, dist));
	VERTEX = vat_position(UV2, INSTANCE_CUSTOM, dist) * far_scale_at(dist);
}

void fragment() {
	vec4 c = mix(day_colour, night_colour, clamp(w3d_night, 0.0, 1.0));
	// Lift the Style 2 night silhouette above pale thatch.
	float night_value = 1.8 * max(wardrobe_value, 0.85);
	ALBEDO = c.rgb * (distant_only ? mix(1.0, night_value, w3d_night) : 1.0);
	ALPHA = c.a * (distant_only ? outline_weight : 1.0);
}

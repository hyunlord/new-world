// The opaque body marks visible pixels. Hidden people can still show through walls
// and overlap each other without painting on any visible body.
shader_type spatial;
render_mode unshaded, depth_test_inverted, depth_draw_never, blend_mix, cull_back, shadows_disabled, fog_disabled;
stencil_mode read, compare_not_equal, 27;

#include "res://shaders/world3d/vat_anim.gdshaderinc"
global uniform float w3d_night;
uniform vec4 day_colour : source_color = vec4(0.14, 0.11, 0.09, 0.8);
uniform vec4 night_colour : source_color = vec4(0.57, 0.65, 0.66, 0.54);
varying flat float map_readability;

void vertex() {
	float dist = length((MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz - CAMERA_POSITION_WORLD);
	VERTEX = vat_position(UV2, INSTANCE_CUSTOM) * far_scale_at(dist);
	map_readability = smoothstep(far_scale.x * 0.75, far_scale.x, dist);
}

void fragment() {
	// A stronger blue-grey cue is needed once a silhouette is only a few pixels wide.
	vec4 map_night = vec4(night_colour.rgb * mix(1.0, 1.55, map_readability),
		mix(night_colour.a, 0.70, map_readability));
	vec4 c = mix(day_colour, map_night, clamp(w3d_night, 0.0, 1.0));
	ALBEDO = c.rgb;
	ALPHA = c.a;
}

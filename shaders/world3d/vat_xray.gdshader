// vat_xray.gdshader — the see-through pass of the VAT people (after the outline pass; scripts/world3d/vat3d.gd).
// Where a roof, wall or tree stands more than `min_gap` metres in front of the animated body, the body is drawn as a
// flat translucent silhouette (light at night, dark by day), so people indoors or under canopy stay findable (the 2D
// view drew everyone on top). The person's own parts never count as an occluder (they are within `min_gap`).
shader_type spatial;
render_mode unshaded, depth_test_disabled, depth_draw_never, blend_mix, cull_back, shadows_disabled, specular_disabled, fog_disabled;

#include "res://shaders/world3d/vat_anim.gdshaderinc"
global uniform float w3d_night;
uniform sampler2D depth_tex : hint_depth_texture, filter_nearest;
uniform float min_gap = 0.3;
uniform vec4 day_colour : source_color = vec4(0.14, 0.11, 0.09, 0.8);
uniform vec4 night_colour : source_color = vec4(0.57, 0.65, 0.66, 0.54);
varying flat float map_readability;

void vertex() {
	float dist = length((MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz - CAMERA_POSITION_WORLD);
	VERTEX = vat_position(UV2, INSTANCE_CUSTOM) * far_scale_at(dist);
	map_readability = smoothstep(far_scale.x * 0.75, far_scale.x, dist);
}

void fragment() {
	float scene = texture(depth_tex, SCREEN_UV).r;
	vec4 view = INV_PROJECTION_MATRIX * vec4(SCREEN_UV * 2.0 - 1.0, scene, 1.0);
	float scene_z = -view.z / view.w;
	if (-VERTEX.z - scene_z < min_gap) {
		discard;
	}
	// A stronger blue-grey cue is needed once a silhouette is only a few pixels wide.
	vec4 map_night = vec4(night_colour.rgb * mix(1.0, 1.55, map_readability),
		mix(night_colour.a, 0.70, map_readability));
	vec4 c = mix(day_colour, map_night, clamp(w3d_night, 0.0, 1.0));
	ALBEDO = c.rgb;
	ALPHA = c.a;
}

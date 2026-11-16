// vat_outline.gdshader — the outline pass of the VAT people (next_pass of vat.gdshader, scripts/world3d/vat3d.gd).
// The same animated body pushed out by `outline_px` screen pixels from its axis and drawn back faces only: a dark rim by
// day and a light halo at night (global `w3d_night` from sky3d.gd), so a person stays findable on any ground and in the
// dark (the 2D view's ring and night halo).
shader_type spatial;
render_mode unshaded, cull_front, shadows_disabled, specular_disabled, fog_disabled;

#include "res://shaders/world3d/vat_anim.gdshaderinc"
global uniform float w3d_night;
uniform float outline_px = 2.0;
// Wider beyond far_scale.x (up to outline_far_px at twice that distance): small far figures need a thicker rim.
uniform float outline_far_px = 4.0;
uniform float body_height = 1.7;
uniform vec3 day_colour : source_color = vec3(0.14, 0.11, 0.09);
uniform vec3 night_colour : source_color = vec3(0.91, 0.89, 0.81);

void vertex() {
	float dist = length((MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz - CAMERA_POSITION_WORLD);
	vec3 p = vat_position(UV2, INSTANCE_CUSTOM) * far_scale_at(dist);
	// One screen pixel in metres at this distance.
	float px = 2.0 * dist / (PROJECTION_MATRIX[1][1] * VIEWPORT_SIZE.y);
	float s = far_scale_at(dist);
	vec3 axis = vec3(0.0, clamp(p.y, 0.25 * s, (body_height - 0.25) * s), 0.0);
	vec3 dir = p - axis;
	float rim = mix(outline_px, outline_far_px, smoothstep(far_scale.x, far_scale.x * 2.0, dist));
	VERTEX = p + (length(dir) > 1.0e-4 ? normalize(dir) : vec3(0.0, 1.0, 0.0)) * rim * px;
	// Orthographic projection = a directional shadow pass: the outline must not cast (its width is in screen pixels).
	if (PROJECTION_MATRIX[3][3] > 0.5) {
		VERTEX = vec3(0.0);
	}
}

void fragment() {
	ALBEDO = mix(day_colour, night_colour, clamp(w3d_night, 0.0, 1.0));
}

// contact_blob.gdshader — the soft dark disc under each person (scripts/world3d/people3d.gd): a radial falloff on a
// ground quad, so people read against any ground and in shade.
shader_type spatial;
render_mode unshaded, blend_mix, depth_draw_never, cull_disabled, shadows_disabled;

uniform float alpha = 0.45;

void fragment() {
	float d = length(UV - vec2(0.5)) * 2.0;
	ALBEDO = vec3(0.0);
	ALPHA = alpha * (1.0 - smoothstep(0.35, 1.0, d));
}

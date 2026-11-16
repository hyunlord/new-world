// water.gdshader — the 3D view's water surface (scripts/world3d/terrain3d.gd, M3D-1 scope 3): a translucent sheet
// with two scrolling value-noise ripples bending the normal, a fresnel sheen and a light foam band near the banks
// (where the bed under it is shallow, read from the depth texture).
shader_type spatial;
render_mode blend_mix, depth_draw_opaque, cull_back, specular_schlick_ggx;

global uniform float w3d_time;
uniform vec4 water_colour : source_color = vec4(0.16, 0.36, 0.45, 0.82);
uniform sampler2D depth_tex : hint_depth_texture, filter_linear;

varying vec3 world_pos;

float hash12(vec2 p) {
	vec3 p3 = fract(vec3(p.xyx) * 0.1031);
	p3 += dot(p3, p3.yzx + 33.33);
	return fract((p3.x + p3.y) * p3.z);
}

float vnoise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(hash12(i), hash12(i + vec2(1.0, 0.0)), f.x), mix(hash12(i + vec2(0.0, 1.0)), hash12(i + vec2(1.0, 1.0)), f.x), f.y);
}

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 p = world_pos.xz;
	float t = w3d_time;
	float a = vnoise(p * 1.7 + vec2(t * 0.35, t * 0.21));
	float b = vnoise(p * 3.1 - vec2(t * 0.27, -t * 0.31));
	vec3 nrm = normalize(vec3((a - 0.5) * 0.35 + (b - 0.5) * 0.2, 1.0, (b - 0.5) * 0.35 - (a - 0.5) * 0.2));
	NORMAL = normalize((VIEW_MATRIX * vec4(nrm, 0.0)).xyz);
	float depth = texture(depth_tex, SCREEN_UV).r;
	vec4 ndc = vec4(SCREEN_UV * 2.0 - 1.0, depth, 1.0);
	vec4 view = INV_PROJECTION_MATRIX * ndc;
	view.xyz /= view.w;
	float thickness = clamp(-view.z + VERTEX.z, 0.0, 4.0);
	float foam = 1.0 - smoothstep(0.0, 0.08, thickness);
	vec3 c = mix(water_colour.rgb * 1.3, water_colour.rgb * 0.7, smoothstep(0.0, 1.2, thickness));
	c = mix(c, vec3(0.80, 0.86, 0.86), foam * 0.45);
	ALBEDO = c;
	ALPHA = clamp(water_colour.a * smoothstep(0.0, 0.25, thickness) + foam * 0.3, 0.0, 0.96);
	ROUGHNESS = 0.12;
	METALLIC = 0.0;
	SPECULAR = 0.6;
}

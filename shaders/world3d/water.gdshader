// water.gdshader — the 3D view's water surface (scripts/world3d/terrain3d.gd, M3D-1 scope 3): a translucent sheet
// with two scrolling value-noise ripples bending the normal, a fresnel sheen and a light foam band near the banks
// (where the bed under it is shallow, read from the depth texture).
shader_type spatial;
render_mode blend_mix, depth_draw_opaque, cull_back, specular_schlick_ggx;

global uniform float w3d_time;
uniform vec4 water_colour : source_color = vec4(0.16, 0.36, 0.45, 0.82);
uniform sampler2D depth_tex : hint_depth_texture, filter_linear;
global uniform float w3d_night;

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
	float a = vnoise(p * vec2(0.65, 1.5) + vec2(t * 0.045, t * 0.12));
	float b = vnoise(p * vec2(1.4, 2.6) - vec2(t * 0.035, -t * 0.08));
	vec3 nrm = normalize(vec3((a - 0.5) * 0.13 + (b - 0.5) * 0.08, 1.0, (b - 0.5) * 0.13 - (a - 0.5) * 0.08));
	NORMAL = normalize((VIEW_MATRIX * vec4(nrm, 0.0)).xyz);
	float depth = texture(depth_tex, SCREEN_UV).r;
	vec4 ndc = vec4(SCREEN_UV * 2.0 - 1.0, depth, 1.0);
	vec4 view = INV_PROJECTION_MATRIX * ndc;
	view.xyz /= view.w;
	float thickness = clamp(-view.z + VERTEX.z, 0.0, 4.0);
	float foam = (1.0 - smoothstep(0.025, 0.22, thickness)) * smoothstep(0.18, 0.65, a + b * 0.25);
	float depth_tint = 1.0 - exp(-thickness * 1.15);
	vec3 c = mix(water_colour.rgb * vec3(1.5, 1.35, 1.15), water_colour.rgb * 0.67, depth_tint);
	float fresnel = pow(1.0 - max(dot(NORMAL, VIEW), 0.0), 3.0);
	vec3 sky = mix(vec3(0.38, 0.52, 0.64), vec3(0.045, 0.065, 0.12), w3d_night);
	c = mix(c, sky * (0.92 + 0.12 * a), 0.22 + fresnel * 0.36);
	float strokes = smoothstep(0.71, 0.77, a * 0.65 + b * 0.35);
	c += vec3(0.035, 0.045, 0.05) * strokes * (1.0 - foam);
	c = mix(c, vec3(0.78, 0.83, 0.78), foam * 0.55);
	ALBEDO = c;
	ALPHA = clamp(water_colour.a * smoothstep(0.0, 0.32, thickness) + foam * 0.18, 0.0, 0.94);
	ROUGHNESS = 0.3;
	METALLIC = 0.0;
	SPECULAR = 0.35;
}

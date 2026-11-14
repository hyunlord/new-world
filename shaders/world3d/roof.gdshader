// roof.gdshader — the roofs of the 3D view (scripts/world3d/structures3d.gd). `cover` tints the roof surfaces
// (vertex colour alpha 1; the gable triangles carry the wall colour with alpha 0 and stay untinted); `pattern`
// draws thatch courses (0), bark shingles (1) or planks (2) from world position; `fade` < 1 dissolves the roof with a
// stable screen-space dither; the global `w3d_snow` covers the upper roof faces in winter; (the cutaway), so no transparency sorting is needed. UV.x is 0 for a ridge along x
// and 1 for a ridge along z (shingle and plank lines run down the slope).
shader_type spatial;
render_mode diffuse_lambert, specular_disabled, cull_back;

uniform vec3 cover : source_color = vec3(0.72, 0.6, 0.34);
uniform int pattern = 0;
uniform float fade = 1.0;
global uniform float w3d_snow;

varying vec3 world_pos;
varying float ridge_z;
varying float up;

float bayer4(vec2 p) {
	ivec2 q = ivec2(mod(p, 4.0));
	int idx = q.x + q.y * 4;
	float m[16] = float[](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
	return (m[idx] + 0.5) / 16.0;
}

vec3 srgb_to_linear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	ridge_z = UV.x;
	up = (MODEL_MATRIX * vec4(NORMAL, 0.0)).y;
}

void fragment() {
	if (fade < 0.999 && bayer4(FRAGCOORD.xy) > fade) {
		discard;
	}
	vec3 c = srgb_to_linear(COLOR.rgb);
	if (COLOR.a > 0.5) {
		c *= cover;
		float h = world_pos.y;
		// Along the eaves: x for a ridge along x, z for a ridge along z.
		float along = ridge_z < 0.5 ? world_pos.x : world_pos.z;
		if (pattern == 0) {
			float course = fract(h / 0.16);
			float straw = fract(sin(floor(along * 9.0) * 12.9898 + floor(h / 0.16) * 4.1) * 43758.5453);
			c *= (0.78 + 0.22 * smoothstep(0.0, 0.7, course)) * (0.92 + 0.12 * straw);
		} else if (pattern == 1) {
			float row = floor(h / 0.2);
			float shingle = fract(along / 0.35 + mod(row, 2.0) * 0.5);
			c *= (0.8 + 0.2 * smoothstep(0.0, 0.8, fract(h / 0.2))) * mix(0.8, 1.0, smoothstep(0.0, 0.08, shingle));
		} else {
			float plank = fract(along / 0.28);
			c *= mix(0.78, 1.0, smoothstep(0.0, 0.07, plank) * smoothstep(0.0, 0.07, 1.0 - plank));
		}
	}
	if (COLOR.a > 0.5) {
		c = mix(c, vec3(0.80, 0.85, 0.92), w3d_snow * 0.9 * smoothstep(0.3, 0.7, up));
	}
	ALBEDO = c;
	ROUGHNESS = 0.95;
}

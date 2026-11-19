// Painted static meshes for style s2. Walls clamp at cut height and roofs dither under the same room cutaway.
shader_type spatial;
render_mode diffuse_burley, specular_schlick_ggx;

uniform sampler2D albedo_tex : source_color, filter_linear_mipmap_anisotropic, repeat_disable;
uniform bool wall = false;
uniform float cut_all = 99.0;
uniform vec4 cut_rect = vec4(0.0);
uniform float cut_rect_height = 99.0;
uniform float fade = 1.0;
global uniform float w3d_snow;

varying float cut_top;
varying float up;

void vertex() {
	cut_top = 0.0;
	if (wall) {
		vec3 wp = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
		float limit = cut_all;
		if (wp.x >= cut_rect.x && wp.x <= cut_rect.z && wp.z >= cut_rect.y && wp.z <= cut_rect.w) {
			limit = min(limit, cut_rect_height);
		}
		limit /= length(MODEL_MATRIX[1].xyz);
		cut_top = VERTEX.y > limit ? 1.0 : 0.0;
		VERTEX.y = min(VERTEX.y, limit);
	}
	up = normalize(MODEL_NORMAL_MATRIX * NORMAL).y;
}

void fragment() {
	if (fade < 0.999) {
		ivec2 p = ivec2(mod(FRAGCOORD.xy, 4.0));
		float bayer[16] = float[](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
		if ((bayer[p.x + 4 * p.y] + 0.5) / 16.0 > fade) {
			discard;
		}
	}
	vec3 color = texture(albedo_tex, UV).rgb * mix(1.0, 0.8, cut_top);
	float snow = wall ? 0.0 : w3d_snow * 0.9 * smoothstep(0.3, 0.7, up);
	ALBEDO = mix(color, vec3(0.80, 0.85, 0.92), snow);
	ROUGHNESS = 0.91;
	SPECULAR = 0.24;
}

// Painted static meshes for style s2. Walls clamp at cut height and roofs dither under the same room cutaway.
shader_type spatial;
render_mode diffuse_burley, specular_schlick_ggx;

uniform sampler2D albedo_tex : source_color, filter_linear_mipmap_anisotropic, repeat_disable;
uniform sampler2D normal_tex : hint_normal, filter_linear_mipmap_anisotropic, repeat_disable;
uniform sampler2D rm_tex : filter_linear_mipmap_anisotropic, repeat_disable;
uniform bool mapped_surface = false;
uniform sampler2D log_tile : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform bool mapped_log = false;
uniform bool clean_log = false;
uniform vec3 log_size = vec3(1.0);
uniform vec3 log_origin = vec3(0.0);
uniform bool wall = false;
uniform float wall_base_y = 0.0;
uniform bool raised_wall = false;
uniform vec3 door_opening = vec3(0.0);
uniform float cut_all = 99.0;
uniform vec4 cut_rect = vec4(0.0);
uniform float cut_rect_height = 99.0;
uniform float fade = 1.0;
global uniform float w3d_snow;

varying float cut_top;
varying float up;
varying vec2 local_xy;
varying vec3 timber;
varying float fully_cut;

void vertex() {
	local_xy = VERTEX.xy;
	cut_top = 0.0;
	fully_cut = 0.0;
	if (wall) {
		vec3 wp = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
		float limit = cut_all;
		if (wp.x >= cut_rect.x && wp.x <= cut_rect.z && wp.z >= cut_rect.y && wp.z <= cut_rect.w) {
			limit = min(limit, cut_rect_height);
		}
		if (raised_wall) {
			limit -= INSTANCE_CUSTOM.r * 4.0;
			fully_cut = limit <= 0.0 ? 1.0 : 0.0;
		}
		limit = limit / length(MODEL_MATRIX[1].xyz) + (clean_log ? log_origin.y : wall_base_y);
		cut_top = VERTEX.y > limit ? 1.0 : 0.0;
		VERTEX.y = min(VERTEX.y, limit);
	}
	timber = (VERTEX - log_origin) / log_size;
	up = normalize(MODEL_NORMAL_MATRIX * NORMAL).y;
}

void fragment() {
	if (fully_cut > 0.999) {
		discard;
	}
	if (abs(local_xy.x - door_opening.x) < door_opening.y && local_xy.y < door_opening.z) {
		discard;
	}
	if (fade < 0.999) {
		ivec2 p = ivec2(mod(FRAGCOORD.xy, 4.0));
		float bayer[16] = float[](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
		if ((bayer[p.x + 4 * p.y] + 0.5) / 16.0 > fade) {
			discard;
		}
	}
	vec3 color = texture(albedo_tex, UV).rgb * mix(1.0, 0.8, cut_top);
	if (mapped_log) {
		vec3 strip = texture(log_tile, timber.xy * vec2(0.83, 2.45)).rgb;
		// Keep the baked shading in the seams between the stacked courses.
		color = mix(color, strip * mix(0.83, 1.15, clamp(dot(color, vec3(1.0)), 0.0, 1.0)), 0.74);
	}
	float rough = 0.91;
	if (mapped_surface) {
		NORMAL_MAP = texture(normal_tex, UV).rgb;
		NORMAL_MAP_DEPTH = 1.0;
		rough = texture(rm_tex, UV).g;
	}
	if (clean_log && cut_top > 0.999) {
		vec2 p = timber.xz;
		float flow = p.y * 44.0 + sin(p.x * 6.0 + p.y * 3.0) * 1.08;
		float fibers = sin(flow + sin(flow * 0.47) * 1.9) * 0.5 + 0.5;
		float bark_edge = smoothstep(0.91, 0.99, abs(p.y) * 2.0);
		vec3 heartwood = mix(vec3(0.27, 0.14, 0.065), vec3(0.40, 0.24, 0.12), fibers);
		if (mapped_log) {
			heartwood *= mix(vec3(0.82), vec3(1.23), texture(log_tile, vec2(p.x, p.y * 0.6 + 0.5)).rgb);
		}
		color = mix(heartwood, vec3(0.11, 0.060, 0.030), bark_edge);
		NORMAL = normalize((VIEW_MATRIX * vec4(0.0, 1.0, 0.0, 0.0)).xyz);
		NORMAL_MAP_DEPTH = 0.0;
		rough = 0.84;
	}
	float snow = wall ? 0.0 : w3d_snow * 0.9 * smoothstep(0.3, 0.7, up);
	ALBEDO = mix(color, vec3(0.80, 0.85, 0.92), snow);
	ROUGHNESS = rough;
	SPECULAR = 0.24;
}

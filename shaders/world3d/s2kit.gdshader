// Painted static meshes for style s2. Walls clamp at cut height and roofs dither under the same room cutaway.
// `straw_part` (bindings3d s2kit.straw_parts) re-dresses the rake straw of a part such as the gable trim as
// bundled thatch: the kit thatch tile laid down the rake (mesh z/y plane; `straw_rake` = apex y, corner z,
// corner y, band depth in mesh units), with course shadows stepping down the slope and a strand normal, so the
// stretched atlas bundles no longer read as planks. The baked atlas cannot separate straw from wood by colour.
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
uniform bool straw_part = false;
uniform vec4 straw_rake = vec4(1.0, 1.0, 0.0, 0.1);
uniform sampler2D straw_tex : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D straw_normal_tex : hint_normal, filter_linear_mipmap_anisotropic, repeat_enable;
global uniform float w3d_snow;

varying float cut_top;
varying float up;
varying vec2 local_xy;
varying vec3 timber;
varying float fully_cut;
varying vec2 rake_pos;
varying vec2 rake_mesh;
varying vec2 rake_slope;

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
	vec3 part_scale = vec3(length(MODEL_MATRIX[0].xyz), length(MODEL_MATRIX[1].xyz), length(MODEL_MATRIX[2].xyz));
	rake_pos = VERTEX.zy * part_scale.zy;
	rake_mesh = VERTEX.zy;
	rake_slope = vec2(straw_rake.y, straw_rake.x - straw_rake.z) * part_scale.zy;
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
	if (straw_part) {
		// The band under the rake line (apex to corner), above the tie beam and below the crossed horns.
		float line_y = straw_rake.x - (straw_rake.x - straw_rake.z) * abs(rake_mesh.x) / straw_rake.y;
		float below = line_y - rake_mesh.y;
		float straw_mask = smoothstep(-0.03, 0.0, below) * (1.0 - smoothstep(straw_rake.w * 0.75, straw_rake.w, below))
			* smoothstep(straw_rake.z - 0.03, straw_rake.z, rake_mesh.y) * (1.0 - smoothstep(straw_rake.x - 0.02, straw_rake.x + 0.01, rake_mesh.y));
		vec2 down = normalize(vec2(sign(rake_pos.x) * rake_slope.x, -rake_slope.y));
		// Across the rake, down the rake, in metres; courses of ~0.17 m step down the slope.
		vec2 q = vec2(dot(rake_pos, vec2(-down.y, down.x)) * sign(rake_pos.x), dot(rake_pos, down));
		float wobble = sin(q.x * 21.0) * 0.035 + sin(q.x * 57.0 + q.y * 3.0) * 0.012;
		vec2 grain_uv = vec2(q.x * 1.18, q.y * 1.73 + wobble);
		vec3 straw = texture(straw_tex, grain_uv).rgb;
		float course = fract(q.y / 0.17 + wobble * 4.0);
		float layering = mix(0.5, 1.08, smoothstep(0.03, 0.35, course));
		float bundle = 0.9 + 0.1 * sin(q.x * 37.0 + floor(q.y / 0.17) * 2.3);
		color = mix(color, straw * 1.25 * layering * bundle, straw_mask);
		// Strand normal: the tile's normal map tilted along the rake axes of this part.
		vec2 detail = texture(straw_normal_tex, grain_uv).rg * 2.0 - 1.0;
		vec2 inv = vec2(1.0) / max(rake_slope / vec2(straw_rake.y, straw_rake.x - straw_rake.z), vec2(0.0001));
		vec3 along = normalize((MODEL_MATRIX * vec4(0.0, down.y * inv.y, down.x * inv.x, 0.0)).xyz);
		vec3 side = normalize((MODEL_MATRIX * vec4(0.0, down.x * inv.y, -down.y * inv.x, 0.0)).xyz);
		vec3 tilt = (VIEW_MATRIX * vec4(side * detail.x - along * (detail.y + (1.0 - layering) * 0.9), 0.0)).xyz;
		NORMAL = normalize(NORMAL + tilt * 0.55 * straw_mask);
		NORMAL_MAP_DEPTH = mapped_surface ? 1.0 - straw_mask : 0.0;
		rough = mix(rough, 0.95, straw_mask);
	}
	float snow = wall ? 0.0 : w3d_snow * 0.9 * smoothstep(0.3, 0.7, up);
	ALBEDO = mix(color, vec3(0.80, 0.85, 0.92), snow);
	ROUGHNESS = rough;
	SPECULAR = 0.24;
}

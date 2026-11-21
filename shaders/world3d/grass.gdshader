// Alpha-cutout ground cover. Keep roots planted; fade by shrinking the entire card before culling it.
shader_type spatial;
render_mode cull_disabled, diffuse_lambert, specular_disabled, shadows_disabled;

global uniform float w3d_snow;
global uniform float w3d_time;
global uniform vec3 w3d_focus;
uniform sampler2D cards : filter_linear_mipmap, repeat_disable;
uniform sampler2D path_tex : filter_nearest, repeat_disable;
uniform sampler2D biome_tex : filter_nearest, repeat_disable;
uniform sampler2D built_tex : filter_nearest, repeat_disable;
uniform vec2 map_size;
uniform vec3 shade : source_color;
uniform vec3 root_shade : source_color;
uniform vec3 winter_shade : source_color;
uniform float radius;
uniform float height;
uniform float winter_height = 0.45;
uniform float clearing_limit = 0.48;
uniform float wind_strength = 0.21;
uniform float wind_speed = 1.25;
uniform float cutoff = 0.38;
uniform bool s2 = false;
varying float blade_y;

void vertex() {
	vec3 origin = MODEL_MATRIX[3].xyz;
	vec2 uv = (floor(origin.xz) + 0.5) / map_size;
	float fade = 1.0 - smoothstep(radius * 0.66, radius, distance(origin.xz, w3d_focus.xz));
	float path = textureLod(path_tex, uv, 0.0).r;
	float clearing = textureLod(biome_tex, uv, 0.0).g;
	vec3 built = textureLod(built_tex, uv, 0.0).rgb;
	blade_y = VERTEX.y;
	float phase = dot(origin.xz, vec2(0.49, 0.63));
	float wind = sin(w3d_time * wind_speed + phase) * wind_strength * VERTEX.y * VERTEX.y;
	VERTEX = (VERTEX + vec3(wind, 0.0, 0.0)) * vec3(1.0, height * mix(1.0, winter_height, w3d_snow), 1.0) * fade;
	bool bare = path > 0.001 || clearing >= clearing_limit;
	if (s2) {
		float stage = path * 255.0;
		float trodden = max(max(clearing * 0.42, stage * 0.65), built.g);
		VERTEX.y *= mix(1.0, 1.5, built.b) * (1.0 - 0.74 * clamp(trodden, 0.0, 1.0));
		bare = stage >= 1.8 || built.g >= 0.9;
	}
	if (bare || built.r > 0.5) {
		// Collapse excluded cards to a degenerate triangle before rasterization.
		VERTEX = vec3(0.0);
	}
	float normal_side = s2 ? 0.18 : 0.35;
	NORMAL = normalize(vec3(NORMAL.x * normal_side, 1.0, NORMAL.z * normal_side));
}

void fragment() {
	vec4 texel = texture(cards, UV);
	if (texel.a < cutoff) {
		discard;
	}
	vec3 leaf = mix(root_shade, shade, clamp(blade_y, 0.0, 1.0));
	float texture_value = 0.70 + texel.r * 0.45;
	if (s2) {
		leaf = mix(root_shade, shade, smoothstep(0.20, 0.95, blade_y) * mix(0.70, 1.0, COLOR.g));
		texture_value = mix(0.92, 1.06, texel.r);
	}
	ALBEDO = mix(leaf, winter_shade, w3d_snow) * texture_value * COLOR.r;
	ROUGHNESS = 0.92;
}

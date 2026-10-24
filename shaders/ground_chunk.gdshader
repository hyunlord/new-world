// ground_chunk.gdshader — one baked ground chunk over the water (M-ART2, MART2_DECISIONS D5; M-ART2.1 D3).
// The chunk texture is opaque on land and transparent on water and its dithered bank; beneath it this shader draws
// the seamless 05 water texture sampled with world offsets (so it continues across chunks) and a gentle drift
// (`drift_hz` cycles per second around a `drift_px` circle), attenuated by water_gain to a dark-blue mean.
// World-space snow and cached occupancy masks add soft ground cover and tree-base/wall drifts.
// Below 1× land and water mix toward their map-mode colours. World position comes from MODEL_MATRIX in vertex().
shader_type canvas_item;

uniform sampler2D water_tex : repeat_enable, filter_nearest;
uniform float water_period = 256.0;
uniform float drift_px = 6.0;
uniform float drift_hz = 0.05;
// D19: this terrain-only palette is independent of the reduced ground texture and all standing sprites.
uniform sampler2D terrain_tint_tex : filter_linear;
uniform float chunk_tiles = 16.0;
uniform float map_tint_strength = 0.0;
uniform vec3 water_colour = vec3(0.015, 0.05, 0.12);
uniform vec3 water_gain = vec3(1.0);
uniform float water_map_tint_strength = 0.3;
uniform sampler2D snow_mask_tex : filter_nearest;
uniform float snow_alpha = 0.0;
uniform float snow_feature_px = 512.0;
uniform float snow_drift_strength = 0.7;
uniform vec3 snow_colour = vec3(0.78, 0.82, 0.86);

varying vec2 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 0.0, 1.0)).xy;
}

float snow_hash(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

// Smooth world-space cells: neither the texture period nor chunk boundaries restart the snow.
float snow_field(vec2 p) {
	vec2 cell = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(snow_hash(cell), snow_hash(cell + vec2(1.0, 0.0)), f.x),
		mix(snow_hash(cell + vec2(0.0, 1.0)), snow_hash(cell + vec2(1.0)), f.x), f.y);
}

float snow_cover(vec2 uv) {
	vec2 cell = floor(uv * chunk_tiles);
	vec2 local = fract(uv * chunk_tiles);
	float drift = 0.0;
	for (int y = -1; y <= 1; y++) {
		for (int x = -1; x <= 1; x++) {
			vec2 neighbor = vec2(float(x), float(y));
			vec3 mask = texture(snow_mask_tex, (cell + neighbor + vec2(1.5)) / (chunk_tiles + 2.0)).rgb;
			vec2 relative = local - neighbor;
			float tree = 1.0 - smoothstep(0.0, 1.0, length((relative - vec2(0.5, 0.67)) / vec2(0.38, 0.24)));
			float wall = 1.0 - smoothstep(0.0, 0.3, length(max(abs(relative - vec2(0.5)) - vec2(0.5), vec2(0.0))));
			drift = max(drift, max(tree * mask.r, wall * mask.g));
		}
	}
	float exposed = texture(snow_mask_tex, (cell + vec2(1.5)) / (chunk_tiles + 2.0)).b;
	float blanket = mix(0.15, 0.65, snow_field(world_pos / max(512.0, snow_feature_px)));
	return min(0.45, snow_alpha) * exposed * clamp(blanket + drift * snow_drift_strength, 0.0, 1.0);
}

void fragment() {
	vec4 land = texture(TEXTURE, UV);
	vec2 tint_uv = (UV * chunk_tiles + vec2(1.0)) / (chunk_tiles + 2.0);
	vec4 tint = texture(terrain_tint_tex, tint_uv);
	vec2 tile_uv = (floor(UV * chunk_tiles) + vec2(1.5)) / (chunk_tiles + 2.0);
	float tint_allowed = texture(terrain_tint_tex, tile_uv).a;
	land.rgb = mix(land.rgb, tint.rgb, map_tint_strength * tint_allowed);
	if (snow_alpha > 0.0) {
		land.rgb = mix(land.rgb, snow_colour, snow_cover(UV));
	}
	float phase = 6.28318530718 * drift_hz * TIME;
	vec2 drift = drift_px * vec2(cos(phase), sin(phase));
	vec4 water = texture(water_tex, (floor(world_pos) + drift) / water_period);
	water.rgb = clamp(water.rgb * water_gain, vec3(0.0), vec3(1.0));
	water.rgb = mix(water.rgb, water_colour, map_tint_strength * water_map_tint_strength);
	COLOR = vec4(mix(water.rgb, land.rgb, land.a), 1.0);
}

// ground_chunk.gdshader — one baked ground chunk over the water (M-ART2, MART2_DECISIONS D5).
// The chunk texture is opaque on land and transparent on water and its dithered bank; beneath it this shader draws
// the seamless water texture sampled with world offsets (so it continues across chunks) and a gentle drift
// (`drift_hz` cycles per second around a `drift_px` circle). World position comes from MODEL_MATRIX in vertex().
shader_type canvas_item;

uniform sampler2D water_tex : repeat_enable, filter_nearest;
uniform float water_period = 256.0;
uniform float drift_px = 6.0;
uniform float drift_hz = 0.05;
// D19: this terrain-only palette is independent of the reduced ground texture and all standing sprites.
uniform sampler2D terrain_tint_tex : filter_linear;
uniform float chunk_tiles = 16.0;
uniform float map_tint_strength = 0.0;
uniform vec3 water_colour = vec3(0.42, 0.62, 0.76);
uniform float water_map_tint_strength = 0.3;

varying vec2 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 0.0, 1.0)).xy;
}

void fragment() {
	vec4 land = texture(TEXTURE, UV);
	vec2 tint_uv = (UV * chunk_tiles + vec2(1.0)) / (chunk_tiles + 2.0);
	vec4 tint = texture(terrain_tint_tex, tint_uv);
	vec2 tile_uv = (floor(UV * chunk_tiles) + vec2(1.5)) / (chunk_tiles + 2.0);
	float tint_allowed = texture(terrain_tint_tex, tile_uv).a;
	land.rgb = mix(land.rgb, tint.rgb, map_tint_strength * tint_allowed);
	float phase = 6.28318530718 * drift_hz * TIME;
	vec2 drift = drift_px * vec2(cos(phase), sin(phase));
	vec4 water = texture(water_tex, (floor(world_pos) + drift) / water_period);
	water.rgb = mix(water.rgb, water_colour, map_tint_strength * water_map_tint_strength);
	COLOR = vec4(mix(water.rgb, land.rgb, land.a), 1.0);
}

// ground_chunk.gdshader — one baked ground chunk over the water (M-ART2, MART2_DECISIONS D5).
// The chunk texture is opaque on land and transparent on water and its dithered bank; beneath it this shader draws
// the seamless water texture sampled with world offsets (so it continues across chunks) and a gentle drift
// (`drift_hz` cycles per second around a `drift_px` circle). World position comes from MODEL_MATRIX in vertex().
shader_type canvas_item;

uniform sampler2D water_tex : repeat_enable, filter_nearest;
uniform float water_period = 256.0;
uniform float drift_px = 6.0;
uniform float drift_hz = 0.05;

varying vec2 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 0.0, 1.0)).xy;
}

void fragment() {
	vec4 land = texture(TEXTURE, UV);
	float phase = 6.28318530718 * drift_hz * TIME;
	vec2 drift = drift_px * vec2(cos(phase), sin(phase));
	vec4 water = texture(water_tex, (floor(world_pos) + drift) / water_period);
	COLOR = vec4(mix(water.rgb, land.rgb, land.a), 1.0);
}

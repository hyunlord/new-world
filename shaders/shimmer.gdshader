shader_type canvas_item;

uniform float offset : hint_range(0.0, 1.0) = 0.0;
uniform vec4 base_color : source_color = vec4(0.15, 0.15, 0.20, 1.0);
uniform vec4 shimmer_color : source_color = vec4(0.25, 0.25, 0.30, 1.0);
uniform float speed : hint_range(0.1, 4.0) = 0.6;

void fragment() {
	float phase = fract(TIME * speed + offset);
	float shimmer = smoothstep(phase - 0.15, phase, UV.x)
		* (1.0 - smoothstep(phase, phase + 0.15, UV.x));
	COLOR = mix(base_color, shimmer_color, shimmer);
}

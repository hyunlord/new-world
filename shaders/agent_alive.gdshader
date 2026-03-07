shader_type canvas_item;

uniform float breath_amplitude : hint_range(0.0, 3.0) = 0.8;
uniform float breath_speed : hint_range(0.5, 5.0) = 1.5;
uniform float agent_offset = 0.0;
uniform float speed_factor : hint_range(0.0, 2.0) = 1.0;

void vertex() {
	float sway_strength = smoothstep(0.3, 0.0, speed_factor);
	float breath = sin((TIME * breath_speed + agent_offset) * 6.28318530718) * breath_amplitude * sway_strength;
	float lean = sin((TIME * breath_speed * 0.7 + agent_offset + 2.0) * 6.28318530718) * breath_amplitude * 0.3 * sway_strength;
	VERTEX.y += breath;
	VERTEX.x += lean;
}

void fragment() {
	COLOR = texture(TEXTURE, UV);
}

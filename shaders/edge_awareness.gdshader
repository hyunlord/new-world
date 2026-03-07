shader_type canvas_item;

uniform float intensity_top : hint_range(0.0, 1.0) = 0.0;
uniform float intensity_bottom : hint_range(0.0, 1.0) = 0.0;
uniform float intensity_left : hint_range(0.0, 1.0) = 0.0;
uniform float intensity_right : hint_range(0.0, 1.0) = 0.0;
uniform vec4 alert_color : source_color = vec4(0.9, 0.15, 0.1, 1.0);
uniform float pulse_speed : hint_range(0.5, 5.0) = 2.0;
uniform float gradient_depth : hint_range(0.05, 0.3) = 0.12;

void fragment() {
	float d_top = UV.y;
	float d_bottom = 1.0 - UV.y;
	float d_left = UV.x;
	float d_right = 1.0 - UV.x;

	float g_top = smoothstep(gradient_depth, 0.0, d_top) * intensity_top;
	float g_bottom = smoothstep(gradient_depth, 0.0, d_bottom) * intensity_bottom;
	float g_left = smoothstep(gradient_depth, 0.0, d_left) * intensity_left;
	float g_right = smoothstep(gradient_depth, 0.0, d_right) * intensity_right;

	float glow = max(max(g_top, g_bottom), max(g_left, g_right));
	float pulse = 0.6 + 0.4 * sin(TIME * pulse_speed * 6.28318530718);
	glow *= pulse;

	COLOR = vec4(alert_color.rgb, glow * alert_color.a * 0.7);
}

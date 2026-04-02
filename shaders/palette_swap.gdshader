shader_type canvas_item;

uniform sampler2D palette_lut : filter_nearest, repeat_disable;
uniform float hair_index : hint_range(0.0, 7.0, 1.0) = 0.0;
uniform float body_index : hint_range(0.0, 3.0, 1.0) = 0.0;
uniform float skin_index : hint_range(0.0, 7.0, 1.0) = 0.0;

vec2 palette_uv(float row_index, float column_index) {
	return vec2((column_index + 0.5) / 8.0, (row_index + 0.5) / 3.0);
}

void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	if (tex.a <= 0.01) {
		COLOR = tex;
	} else {

	float row_selector = tex.g;
	float row_index = 1.0;
	float column_index = body_index;

	if (row_selector < 0.25) {
		row_index = 0.0;
		column_index = hair_index;
	} else if (row_selector < 0.75) {
		row_index = 1.0;
		column_index = body_index;
	} else {
		row_index = 2.0;
		column_index = skin_index;
	}

	vec4 palette_color = texture(palette_lut, palette_uv(row_index, column_index));
	COLOR = vec4(palette_color.rgb, tex.a);
	}
}

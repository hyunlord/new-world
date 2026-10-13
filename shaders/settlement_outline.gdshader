shader_type canvas_item;
render_mode unshaded;
uniform vec4 outline_color : source_color = vec4(1.0);
uniform vec4 atlas_rect;
float within_frame(vec2 uv) {
	return float(uv.x >= atlas_rect.x && uv.y >= atlas_rect.y && uv.x < atlas_rect.z && uv.y < atlas_rect.w);
}
void fragment() {
	vec2 left = UV - vec2(TEXTURE_PIXEL_SIZE.x, 0.0);
	vec2 right = UV + vec2(TEXTURE_PIXEL_SIZE.x, 0.0);
	vec2 up = UV - vec2(0.0, TEXTURE_PIXEL_SIZE.y);
	vec2 down = UV + vec2(0.0, TEXTURE_PIXEL_SIZE.y);
	float a = texture(TEXTURE, UV).a * within_frame(UV);
	float edge = max(max(texture(TEXTURE, left).a * within_frame(left), texture(TEXTURE, right).a * within_frame(right)), max(texture(TEXTURE, up).a * within_frame(up), texture(TEXTURE, down).a * within_frame(down)));
	COLOR = vec4(outline_color.rgb, max(0.0, edge - a) * outline_color.a);
}

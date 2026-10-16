shader_type canvas_item;
void fragment() {
	vec4 source = texture(TEXTURE, UV);
	float grey = dot(source.rgb, vec3(0.2126, 0.7152, 0.0722));
	COLOR = vec4(mix(source.rgb, vec3(grey), 0.6), source.a * 0.85);
}

shader_type canvas_item;

uniform sampler2D region_texture : filter_nearest, repeat_disable;
uniform sampler2D ramp_texture : filter_nearest, repeat_disable;
varying flat vec3 variant_codes;

void vertex() {
	variant_codes = floor(COLOR.rgb * 15.0 + vec3(0.5));
}

void fragment() {
	vec4 original = texture(TEXTURE, UV);
	vec4 indices = texture(region_texture, UV);
	int region = int(floor(indices.r * 255.0 + 0.5));
	float shade = clamp(floor(indices.g * 255.0 + 0.5), 0.0, 2.0);
	float row = 0.0;
	if (region == 2) {
		row = clamp(variant_codes.r, 0.0, 3.0);
	} else if (region == 3) {
		row = 4.0 + clamp(variant_codes.g, 0.0, 4.0);
	} else if (region == 4) {
		row = 9.0 + clamp(variant_codes.b, 0.0, 5.0);
	}
	if (region >= 2 && region <= 4) {
		COLOR = vec4(texture(ramp_texture, vec2((shade + 0.5) / 3.0, (row + 0.5) / 15.0)).rgb, original.a);
	} else {
		COLOR = original;
	}
}

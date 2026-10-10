shader_type canvas_item;

uniform sampler2D region_texture : filter_nearest, repeat_disable;
uniform sampler2D ramp_texture : filter_nearest, repeat_disable;
varying flat vec2 variant_codes;   // r = tone 0..2, g = hair 0..4

void vertex() {
	variant_codes = floor(COLOR.rg * 15.0 + vec2(0.5));
}

void fragment() {
	vec4 original = texture(TEXTURE, UV);
	int region = int(floor(texture(region_texture, UV).r * 255.0 + 0.5));
	if (region == 3) {
		float shade = clamp(floor(texture(region_texture, UV).g * 255.0 + 0.5), 0.0, 2.0);
		float row = 4.0 + clamp(variant_codes.g, 0.0, 4.0);
		COLOR = vec4(texture(ramp_texture, vec2((shade + 0.5) / 3.0, (row + 0.5) / 15.0)).rgb, original.a);
	} else {
		float luma = dot(original.rgb, vec3(0.2126, 0.7152, 0.0722));
		float tone = variant_codes.r < 0.5 ? 0.90 : (variant_codes.r < 1.5 ? 1.0 : 1.08);
		COLOR = vec4(luma < 0.10 ? original.rgb : clamp(original.rgb * tone, 0.0, 1.0), original.a);
	}
}

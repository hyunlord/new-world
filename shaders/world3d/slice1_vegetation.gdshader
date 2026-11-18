shader_type spatial;
render_mode cull_disabled;

global uniform float w3d_time;
uniform sampler2D colour_map : source_color, filter_linear_mipmap_anisotropic;
uniform sampler2D normal_map : hint_normal, filter_linear_mipmap_anisotropic;
uniform sampler2D rough_map : hint_default_white, filter_linear_mipmap_anisotropic;
uniform sampler2D cutout_map : hint_default_white, filter_linear_mipmap_anisotropic;
uniform bool leaf_card = false;
uniform float plant_height = 5.0;
uniform float gust_amplitude = 0.11;

varying vec3 instance_tint;

void vertex() {
	// Keep every material on a plant in phase and the roots fixed.
	vec2 origin = MODEL_MATRIX[3].xz;
	float wave = w3d_time * 1.3 + dot(origin, vec2(0.23, 0.31));
	float height = clamp(VERTEX.y / plant_height, 0.0, 1.0);
	VERTEX.xz += height * height * gust_amplitude * vec2(sin(wave), cos(wave * 0.71));
	instance_tint = COLOR.rgb;
}

void fragment() {
	if (leaf_card && texture(cutout_map, UV).r < 0.45) {
		discard;
	}
	ALBEDO = texture(colour_map, UV).rgb * instance_tint;
	ROUGHNESS = texture(rough_map, UV).r;
	NORMAL_MAP = texture(normal_map, UV).rgb;
	if (leaf_card) {
		BACKLIGHT = ALBEDO * 0.25;
	}
}

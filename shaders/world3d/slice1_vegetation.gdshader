shader_type spatial;
render_mode cull_disabled;

global uniform float w3d_time;
global uniform vec3 w3d_camera;
global uniform vec3 w3d_focus;
global uniform float w3d_xray;
uniform sampler2D colour_map : source_color, filter_linear_mipmap_anisotropic;
uniform sampler2D normal_map : hint_normal, filter_linear_mipmap_anisotropic;
uniform sampler2D rough_map : hint_default_white, filter_linear_mipmap_anisotropic;
uniform sampler2D cutout_map : hint_default_white, filter_linear_mipmap_anisotropic;
uniform bool leaf_card = false;
uniform bool rgba_card = false;
uniform bool xray = false;
uniform float detail = 1.0;
uniform float plant_height = 5.0;
uniform float gust_amplitude = 0.11;

varying vec3 instance_tint;
varying vec3 world_pos;

void vertex() {
	// Keep every material on a plant in phase and the roots fixed.
	vec2 origin = MODEL_MATRIX[3].xz;
	float wave = w3d_time * 1.3 + dot(origin, vec2(0.23, 0.31));
	float height = clamp(VERTEX.y / plant_height, 0.0, 1.0);
	VERTEX.xz += height * height * gust_amplitude * vec2(sin(wave), cos(wave * 0.71));
	instance_tint = COLOR.rgb;
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec4 colour = texture(colour_map, UV);
	if ((leaf_card && texture(cutout_map, UV).r < 0.45) || (rgba_card && colour.a < 0.35)) {
		discard;
	}
	if (xray && w3d_xray > 0.0) {
		vec3 axis = w3d_focus - w3d_camera;
		float len = length(axis);
		if (len > 0.01) {
			vec3 dir = axis / len;
			float t = dot(world_pos - w3d_camera, dir);
			float d = length(world_pos - (w3d_camera + dir * t));
			float keep = smoothstep(w3d_xray * 0.7, w3d_xray * 1.15, d);
			ivec2 q = ivec2(mod(FRAGCOORD.xy, 4.0));
			float threshold[16] = float[](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
			if (t < len - 0.8 && (threshold[q.x + q.y * 4] + 0.5) / 16.0 > keep) {
				discard;
			}
		}
	}
	ALBEDO = colour.rgb * instance_tint;
	ROUGHNESS = rgba_card ? 0.94 : mix(0.94, texture(rough_map, UV).r, detail);
	if (!rgba_card) {
		NORMAL_MAP = texture(normal_map, UV).rgb;
		NORMAL_MAP_DEPTH = detail;
	}
	if (leaf_card || rgba_card) {
		BACKLIGHT = ALBEDO * 0.25;
	}
}

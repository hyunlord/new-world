// Rooted wind for existing plant/crop batches and low wild-plant groundcover.
// Detail fades with camera distance/zoom; the primary phase silhouettes remain visible.
shader_type spatial;
render_mode cull_disabled, specular_disabled;

#include "res://shaders/world3d/painterly.gdshaderinc"

global uniform float w3d_time;
global uniform vec3 w3d_camera;
global uniform vec3 w3d_focus;
uniform vec4 albedo : source_color = vec4(1.0);
uniform bool use_vertex_colour = false;
uniform bool vertex_colour_srgb = true;
uniform float roughness = 0.95;
uniform bool has_texture = false;
uniform sampler2D albedo_texture : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform bool detail = false;
uniform float fade_start = 42.0;
uniform float fade_end = 65.0;

varying vec3 world_pos;
varying float visibility;

void vertex() {
	vec3 origin = (MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
	float phase = origin.x * 0.71 + origin.z * 0.43;
	float bend = max(VERTEX.y, 0.0);
	VERTEX.x += sin(w3d_time * 1.2 + phase) * bend * 0.055;
	VERTEX.z += cos(w3d_time * 0.9 + phase * 1.3) * bend * 0.035;
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float distance_fade = max(distance(w3d_camera, world_pos), distance(w3d_camera, w3d_focus));
	visibility = detail ? 1.0 - smoothstep(fade_start, max(fade_start + 0.01, fade_end), distance_fade) : 1.0;
}

void fragment() {
	float dither = fract(dot(floor(FRAGCOORD.xy), vec2(0.754877666, 0.569840296)));
	if (visibility <= dither) {
		discard;
	}
	vec4 c = albedo;
	if (use_vertex_colour) {
		vec3 vertex_tint = COLOR.rgb;
		if (vertex_colour_srgb) {
			vertex_tint = mix(COLOR.rgb / 12.92, pow((COLOR.rgb + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), COLOR.rgb));
		}
		c.rgb *= vertex_tint;
	}
	if (has_texture) {
		c *= texture(albedo_texture, UV);
	}
	ALBEDO = painterly_albedo(c.rgb, world_pos);
	ROUGHNESS = roughness;
}

void light() {
	DIFFUSE_LIGHT += painterly_diffuse(NORMAL, LIGHT, VIEW, ALBEDO, LIGHT_COLOR, ATTENUATION);
}

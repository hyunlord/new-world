// Static imported surfaces: preserve authored colour/roughness and existing surface count.
#include "res://shaders/world3d/material_texture.gdshaderinc"
#include "res://shaders/world3d/painterly.gdshaderinc"
uniform vec4 base_colour : source_color = vec4(1.0);
uniform float texture_tile = -1.0;
uniform float roughness = 0.9;
uniform float metallic = 0.0;
uniform float specular = 0.5;
uniform bool vertex_tint = false;
uniform bool vertex_srgb = false;
varying vec3 material_pos;
varying vec3 material_normal;

void vertex() {
	material_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	material_normal = normalize(MODEL_NORMAL_MATRIX * NORMAL);
}

void fragment() {
	ALBEDO = base_colour.rgb * material_triplanar(material_pos * 0.7, material_normal, texture_tile);
	if (vertex_tint) {
		vec3 tint = COLOR.rgb;
		if (vertex_srgb) {
			tint = mix(tint / 12.92, pow((tint + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), tint));
		}
		ALBEDO *= tint;
	}
	ALBEDO = painterly_albedo(ALBEDO, material_pos);
	vec3 facing_normal = FRONT_FACING ? material_normal : -material_normal;
	NORMAL = normalize((VIEW_MATRIX * vec4(material_bump(material_pos * 0.7, facing_normal, texture_tile), 0.0)).xyz);
	ROUGHNESS = roughness;
	METALLIC = metallic;
	SPECULAR = specular;
}

void light() {
	DIFFUSE_LIGHT += painterly_diffuse(NORMAL, LIGHT, VIEW, ALBEDO, LIGHT_COLOR, ATTENUATION);
	vec3 half_vector = normalize(LIGHT + VIEW);
	float shine = pow(max(dot(NORMAL, half_vector), 0.0), mix(64.0, 8.0, ROUGHNESS));
	SPECULAR_LIGHT += shine * LIGHT_COLOR * ATTENUATION * specular * (1.0 - ROUGHNESS) * 0.12;
}

// Static imported surfaces: preserve authored colour/roughness and existing surface count.
#include "res://shaders/world3d/material_texture.gdshaderinc"
uniform vec4 base_colour : source_color = vec4(1.0);
uniform float texture_tile = -1.0;
uniform float roughness = 0.9;
uniform float metallic = 0.0;
uniform float specular = 0.5;
uniform bool vertex_tint = false;
varying vec3 material_pos;
varying vec3 material_normal;

void vertex() {
	material_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	material_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	ALBEDO = base_colour.rgb * material_triplanar(material_pos * 0.7, material_normal, texture_tile);
	if (vertex_tint) { ALBEDO *= COLOR.rgb; }
	vec3 facing_normal = FRONT_FACING ? material_normal : -material_normal;
	NORMAL = normalize((VIEW_MATRIX * vec4(material_bump(material_pos * 0.7, facing_normal, texture_tile), 0.0)).xyz);
	ROUGHNESS = roughness;
	METALLIC = metallic;
	SPECULAR = specular;
}

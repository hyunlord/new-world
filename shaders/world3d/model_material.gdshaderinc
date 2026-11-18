// Static imported surfaces: preserve authored colour/roughness and existing surface count.
#include "res://shaders/world3d/material_texture.gdshaderinc"
#ifdef SLICE_PBR
#include "res://shaders/world3d/slice_pbr.gdshaderinc"
#endif
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
	vec3 tex_pos = material_pos * (0.7 * material_scale);
#ifdef SLICE_PBR
	float layer = slice_slot(texture_tile);
	vec3 dx = dFdx(tex_pos);
	vec3 dy = dFdy(tex_pos);
	if (layer >= 0.0) {
		vec3 face = normalize(FRONT_FACING ? material_normal : -material_normal);
		SliceSurface surface = slice_triplanar(tex_pos, dx, dy, face, layer);
		ALBEDO = surface.albedo * mix(vec3(1.0), base_colour.rgb * 1.7, 0.25);
		if (vertex_tint) { ALBEDO *= COLOR.rgb; }
		NORMAL = normalize((VIEW_MATRIX * vec4(slice_perturb(face, surface.bump, 0.56), 0.0)).xyz);
		AO = surface.ao;
		ROUGHNESS = surface.roughness;
		METALLIC = metallic;
		SPECULAR = specular;
	} else {
#endif
	vec3 detail = material_triplanar(tex_pos, material_normal, texture_tile);
	ALBEDO = base_colour.rgb * detail;
	if (vertex_tint) { ALBEDO *= COLOR.rgb; }
	vec3 facing_normal = FRONT_FACING ? material_normal : -material_normal;
	vec3 norm = material_bump(tex_pos, facing_normal, texture_tile);
	norm = material_height_normal(material_pos, norm, detail.r);
	NORMAL = normalize((VIEW_MATRIX * vec4(norm, 0.0)).xyz);
	ROUGHNESS = roughness;
	METALLIC = metallic;
	SPECULAR = specular;
#ifdef SLICE_PBR
	}
#endif
}

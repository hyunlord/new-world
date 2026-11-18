// wall.gdshader — walls, doors, fences, floors and construction sites of the 3D view (scripts/world3d/structures3d.gd).
// COLOR is the material colour; UV.x the profile (0 block: running-bond mortar lines; 1 log; 2 palisade; 3 trim),
// UV2.y the ground the piece stands on. The cutaway clamps every vertex to `cut_all` metres above its ground, and
// to `cut_rect_height` inside `cut_rect` (x0, z0, x1, z1) — the walls around a selected room.
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;
#include "res://shaders/world3d/material_texture.gdshaderinc"

uniform float cut_all = 99.0;
uniform vec4 cut_rect = vec4(0.0);
uniform float cut_rect_height = 99.0;

varying vec3 world_pos;
varying float profile;
varying float cut_top;
varying float texture_tile;
varying vec3 texture_normal;

vec3 srgb_to_linear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

void vertex() {
	vec3 wp = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float lim = cut_all;
	if (wp.x >= cut_rect.x && wp.x <= cut_rect.z && wp.z >= cut_rect.y && wp.z <= cut_rect.w) {
		lim = min(lim, cut_rect_height);
	}
	float top = UV2.y + lim;
	cut_top = (VERTEX.y > top) ? 1.0 : 0.0;
	VERTEX.y = min(VERTEX.y, top);
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	profile = UV.x;
	texture_tile = UV2.x;
	texture_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	vec3 c = srgb_to_linear(COLOR.rgb);
	c *= material_triplanar(world_pos * (0.7 * material_scale), texture_normal, texture_tile);
	if (profile < 0.5) {
		// Masonry: courses of 0.22 m, joints every 0.4 m offset by half on alternate courses.
		float course = world_pos.y / 0.22;
		float row = floor(course);
		float along = (world_pos.x + world_pos.z) / 0.4 + mod(row, 2.0) * 0.5;
		float hj = smoothstep(0.0, 0.06, fract(course)) * smoothstep(0.0, 0.06, 1.0 - fract(course));
		float vj = smoothstep(0.0, 0.05, fract(along)) * smoothstep(0.0, 0.05, 1.0 - fract(along));
		float stone = fract(sin(dot(vec2(row, floor(along)), vec2(12.9898, 78.233))) * 43758.5453);
		c *= mix(0.72, 1.0, hj * vj) * (0.9 + 0.12 * stone);
	}
	if (cut_top > 0.5) {
		c *= 0.8;
	}
	ALBEDO = c;
	NORMAL = normalize((VIEW_MATRIX * vec4(material_bump(world_pos * (0.7 * material_scale), texture_normal, texture_tile), 0.0)).xyz);
	ROUGHNESS = 0.92;
}

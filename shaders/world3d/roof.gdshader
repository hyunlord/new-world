// roof.gdshader — the roofs of the 3D view (scripts/world3d/structures3d.gd). `cover` tints the roof surfaces
// (vertex colour alpha 1; the gable triangles carry the wall colour with alpha 0 and stay untinted); `pattern`
// draws thatch courses (0), bark shingles (1) or planks (2) from world position; `fade` < 1 dissolves the roof with a
// stable screen-space dither; the global `w3d_snow` covers the upper roof faces in winter; (the cutaway), so no transparency sorting is needed. UV.x is 0 for a ridge along x
// and 1 for a ridge along z (shingle and plank lines run down the slope).
shader_type spatial;
#ifdef SLICE_PBR
render_mode diffuse_burley, specular_schlick_ggx, cull_back;
#include "res://shaders/world3d/slice_pbr.gdshaderinc"
#else
render_mode diffuse_lambert, specular_disabled, cull_back;
#endif
#include "res://shaders/world3d/material_texture.gdshaderinc"
uniform float texture_tile = -1.0;

uniform vec3 cover : source_color = vec3(0.72, 0.6, 0.34);
uniform int pattern = 0;
uniform float fade = 1.0;
uniform float lay_top = 10000.0;
uniform bool radial_thatch = false;
uniform vec2 cone_center;
uniform bool s2_thatch = false;
uniform float thatch_eave = 0.0;
uniform sampler2D kit_straw : source_color, filter_linear_mipmap, repeat_disable;
uniform vec4 kit_patch;
uniform bool mapped_thatch = false;
uniform bool gable_straw_cards = false;
uniform sampler2D straw_normal : hint_normal, filter_linear_mipmap_anisotropic, repeat_disable;
uniform sampler2D straw_rm : filter_linear_mipmap_anisotropic, repeat_disable;
global uniform float w3d_snow;

varying vec3 world_pos;
varying float ridge_z;
varying float up;
varying vec3 texture_normal;

float bayer4(vec2 p) {
	ivec2 q = ivec2(mod(p, 4.0));
	int idx = q.x + q.y * 4;
	float m[16] = float[](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
	return (m[idx] + 0.5) / 16.0;
}

vec3 srgb_to_linear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

void thatch_surface(vec3 vertex_colour, out vec3 albedo, out vec3 normal, out float roughness) {
	float across = ridge_z < 0.5 ? world_pos.x : world_pos.z;
	float h = world_pos.y - thatch_eave;
	float angle = 0.0;
	if (radial_thatch) {
		vec2 radial = world_pos.xz - cone_center;
		angle = atan(radial.y, radial.x);
		across = angle * 1.67;
	}
	// Weathered bundles have a broad low-frequency shade and finer torn course edges.
	float bundle = sin(across * 29.0 + cos(across * 12.0) * 1.6);
	float drift = sin(across * 4.9 + h * 1.5);
	float fibres = sin(across * 112.0 + h * 12.0 + cos(across * 15.0) * 1.8);
	if (radial_thatch) {
		bundle = sin(angle * 37.0 + cos(angle * 21.0) * 1.7 + h * 1.2);
		drift = sin(angle * 9.0 - h * 2.1);
		fibres = sin(angle * 183.0 - h * 18.0 + cos(angle * 25.0) * 2.0);
	}
	float course = fract(h / 0.29 + bundle * (radial_thatch ? 0.205 : 0.135) + drift * 0.08);
	float shadow = mix(radial_thatch ? 0.84 : 0.68, 1.05, smoothstep(0.05, 0.25, course));
	if (gable_straw_cards) {
		float bent = h * 3.9 + sin(across * 26.0 + h * 4.0) * 0.24 + sin(across * 8.0 - h * 7.0) * 0.22;
		float knots = smoothstep(-0.44, 0.75, sin(across * 30.0 + h * 19.0));
		shadow = mix(1.04, 0.88, (1.0 - smoothstep(0.08, 0.44, fract(bent))) * knots);
	}
	fibres *= 1.0 - smoothstep(0.02, 0.085, fwidth(across));
	float grain_x = radial_thatch ? angle * (34.0 / TAU) : across * 3.35;
	vec2 grain_uv = vec2(grain_x + h * 0.24, h * 1.63 + 0.08 * drift);
	if (mapped_thatch) {
		grain_uv = vec2(across * 1.18 + h * 0.07, -h * 1.73 + 0.046 * drift);
	}
	if (gable_straw_cards) {
		grain_uv += vec2(0.021 * sin(across * 17.0 + h * 3.0), 0.064 * sin(across * 14.0 + h * 10.0));
	}
	vec2 repeat_pos = fract(grain_uv * 0.5) * 2.0 - 1.0;
	vec2 sample_uv = kit_patch.xy + abs(repeat_pos) * kit_patch.zw;
	vec2 dx = dFdx(grain_uv) * kit_patch.zw;
	vec2 dy = dFdy(grain_uv) * kit_patch.zw;
	vec3 straw = textureGrad(kit_straw, sample_uv, dx, dy).rgb;
	float relief = dot(straw, vec3(0.2126, 0.7152, 0.0722));
	float snow = w3d_snow * 0.9 * smoothstep(0.30, 0.75, up);
	vec3 warm = mix(straw * 1.48, cover * vec3(0.79, 0.78, 0.74), 0.22);
	albedo = mix(warm * srgb_to_linear(vertex_colour) * shadow * (0.94 + (radial_thatch ? 0.15 : 0.09) * bundle + 0.08 * drift + 0.045 * fibres),
		vec3(0.80, 0.85, 0.92), snow);
	vec3 face = normalize(texture_normal);
	vec3 relief_normal = material_height_normal(world_pos, face, 0.6 * relief + (gable_straw_cards ? 0.13 : 0.45) * shadow + 0.055 * fibres);
	normal = normalize(mix(face, relief_normal, 2.1));
	roughness = mix(clamp(0.97 - relief * 0.23 + (1.0 - shadow) * 0.08, 0.76, 0.99), 0.96, snow);
	if (mapped_thatch) {
		vec3 along = ridge_z < 0.5 ? vec3(1.0, 0.0, 0.0) : vec3(0.0, 0.0, 1.0);
		vec3 down = cross(face, along);
		down *= down.y > 0.0 ? -1.0 : 1.0;
		vec2 detail = (textureGrad(straw_normal, sample_uv, dx, dy).rg * 2.0 - 1.0) * sign(repeat_pos);
		normal = normalize(normal + (along * detail.x - down * detail.y) * (0.35 * (1.0 - snow)));
		roughness = mix(clamp(textureGrad(straw_rm, sample_uv, dx, dy).g, 0.72, 1.0), 0.96, snow);
	}
}

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	ridge_z = UV.x;
	up = (MODEL_MATRIX * vec4(NORMAL, 0.0)).y;
	texture_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	if (world_pos.y > lay_top) {
		discard;
	}
	if (fade < 0.999 && bayer4(FRAGCOORD.xy) > fade) {
		discard;
	}
	if (s2_thatch && COLOR.a > 0.5) {
		vec3 straw_albedo;
		vec3 thatch_normal;
		float straw_roughness;
		thatch_surface(COLOR.rgb, straw_albedo, thatch_normal, straw_roughness);
		ALBEDO = straw_albedo;
		NORMAL = normalize((VIEW_MATRIX * vec4(thatch_normal, 0.0)).xyz);
		ROUGHNESS = straw_roughness;
	} else {
#ifdef SLICE_PBR
	vec3 tex_pos = world_pos * (0.55 * material_scale);
	vec3 dx = dFdx(tex_pos);
	vec3 dy = dFdy(tex_pos);
	vec3 face = normalize(texture_normal);
	float layer = COLOR.a > 0.5 ? slice_slot(texture_tile) : SLICE_LOG;
	if (layer >= 0.0) {
		SliceSurface surface = slice_triplanar(tex_pos, dx, dy, face, layer);
		float snow_cover = COLOR.a > 0.5 ? w3d_snow * 0.9 * smoothstep(0.3, 0.7, up) : 0.0;
		ALBEDO = mix(surface.albedo, vec3(0.80, 0.85, 0.92), snow_cover);
		NORMAL = normalize((VIEW_MATRIX * vec4(slice_perturb(face, surface.bump, 0.68 * (1.0 - snow_cover)), 0.0)).xyz);
		AO = mix(surface.ao, 1.0, snow_cover);
		ROUGHNESS = mix(surface.roughness, 0.96, snow_cover);
		SPECULAR = 0.32;
	} else {
#endif
	vec3 c = srgb_to_linear(COLOR.rgb);
	float grain = 1.0;
	if (COLOR.a > 0.5) {
		c *= cover;
		float variant = texture_tile == 24.0 ? step(0.5, fract(sin(dot(floor(world_pos.xz / 4.0), vec2(12.9898, 78.233))) * 43758.5453)) : 0.0;
		grain = material_detail(vec2(ridge_z < 0.5 ? world_pos.x : world_pos.z, world_pos.y) * (0.65 * material_scale), texture_tile + variant).r;
		c *= grain;
		float h = world_pos.y;
		// Along the eaves: x for a ridge along x, z for a ridge along z.
		float along = ridge_z < 0.5 ? world_pos.x : world_pos.z;
		if (radial_thatch) {
			vec2 radial = world_pos.xz - cone_center;
			float angle = atan(radial.y, radial.x);
			float bundle = cos(angle * 56.0) * 0.07 + sin(angle * 97.0) * 0.055;
			float strands = cos(angle * 177.0 + h * 3.0) * 0.045;
			float course = smoothstep(0.0, 0.8, fract(h / 0.28 + bundle));
			c *= 0.94 + bundle + strands + 0.1 * course;
		} else if (pattern == 0) {
			float course = fract(h / 0.16);
			float straw = fract(sin(floor(along * 9.0) * 12.9898 + floor(h / 0.16) * 4.1) * 43758.5453);
			c *= (0.78 + 0.22 * smoothstep(0.0, 0.7, course)) * (0.92 + 0.12 * straw);
		} else if (pattern == 1) {
			float row = floor(h / 0.2);
			float shingle = fract(along / 0.35 + mod(row, 2.0) * 0.5);
			c *= (0.8 + 0.2 * smoothstep(0.0, 0.8, fract(h / 0.2))) * mix(0.8, 1.0, smoothstep(0.0, 0.08, shingle));
		} else {
			float plank = fract(along / 0.28);
			c *= mix(0.78, 1.0, smoothstep(0.0, 0.07, plank) * smoothstep(0.0, 0.07, 1.0 - plank));
		}
	}
	if (COLOR.a > 0.5) {
		c = mix(c, vec3(0.80, 0.85, 0.92) * material_detail(world_pos.xz * 0.3, 8.0), w3d_snow * 0.9 * smoothstep(0.3, 0.7, up));
	}
	ALBEDO = c;
	vec3 norm = material_bump(world_pos * (0.65 * material_scale), texture_normal, texture_tile);
	norm = material_height_normal(world_pos, norm, grain);
	NORMAL = normalize((VIEW_MATRIX * vec4(norm, 0.0)).xyz);
	ROUGHNESS = 0.95;
#ifdef SLICE_PBR
	}
#endif
	}
}

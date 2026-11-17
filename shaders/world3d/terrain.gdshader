// terrain.gdshader — the 3D view's ground (scripts/world3d/terrain3d.gd, M3D-1 scope 3/8/9).
// Albedo is the per-corner vertex colour (terrain code colour, clearing, floors, fields) times the season tint,
// broken up by broad painted noise and the shared atlas; slopes turn to triplanar rock and UV.x blends the bank.
// Snow covers upward faces by `snow` (winter). `overlay_tex` is one texel per map
// tile (RGBA fill; alpha 0 = nothing) for the overlays and order feedback; `frame_tex` marks tiles whose outline is
// drawn (R = outline strength) so hover, drag rectangles and refused tiles read as framed tiles.
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;
#include "res://shaders/world3d/material_texture.gdshaderinc"
#include "res://shaders/world3d/painterly.gdshaderinc"
uniform sampler2D biome_tex : filter_nearest, repeat_disable;
uniform float autumn = 0.0;

uniform vec3 season_tint = vec3(1.0);
uniform float snow = 0.0;
uniform vec2 map_size = vec2(192.0, 192.0);
uniform sampler2D overlay_tex : filter_nearest, repeat_disable, source_color;
uniform sampler2D frame_tex : filter_nearest, repeat_disable;
uniform float overlay_on = 0.0;
uniform float grid_on = 0.0;
uniform vec3 cliff_colour : source_color = vec3(0.43, 0.39, 0.34);
uniform vec3 shore_colour : source_color = vec3(0.55, 0.52, 0.45);

varying vec3 world_pos;
varying vec3 world_normal;

float hash12(vec2 p) {
	vec3 p3 = fract(vec3(p.xyx) * 0.1031);
	p3 += dot(p3, p3.yzx + 33.33);
	return fract((p3.x + p3.y) * p3.z);
}

float vnoise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	float a = hash12(i);
	float b = hash12(i + vec2(1.0, 0.0));
	float c = hash12(i + vec2(0.0, 1.0));
	float d = hash12(i + vec2(1.0, 1.0));
	return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

vec3 srgb_to_linear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

vec3 ground_detail(vec2 tile_pos, vec3 p, vec3 normal) {
	vec2 biome = texture(biome_tex, (tile_pos + 0.5) / map_size).rg;
	float tile = floor(biome.r * 255.0 + 0.5);
	float variant = step(0.5, hash12(tile_pos));
	if (tile < 0.5) {
		tile = autumn > 0.5 ? 2.0 : variant;
	} else if (abs(tile - 4.0) < 0.5) {
		tile += variant;
	}
	vec3 detail = normal.y > 0.96 ? material_detail(p.xz * 0.32, tile) : material_triplanar(p * 0.32, normal, tile);
	return biome.g > 0.001 ? mix(detail, material_detail(p.xz * 0.32, 4.0 + variant), biome.g) : detail;
}

vec3 blended_ground(vec3 p, vec3 normal) {
	// The same four neighbouring tiles that supply each vertex's existing colour.
	vec2 base = floor(p.xz - 0.5);
	vec2 f = smoothstep(vec2(0.0), vec2(1.0), fract(p.xz - 0.5));
	return mix(mix(ground_detail(base, p, normal), ground_detail(base + vec2(1.0, 0.0), p, normal), f.x),
		mix(ground_detail(base + vec2(0.0, 1.0), p, normal), ground_detail(base + vec2(1.0), p, normal), f.x), f.y);
}

void fragment() {
	vec3 c = srgb_to_linear(COLOR.rgb) * season_tint;
	vec3 normal = normalize(world_normal);
	c *= blended_ground(world_pos, normal);
	float n = vnoise(world_pos.xz * 0.22) * 0.7 + vnoise(world_pos.xz * 1.5) * 0.3;
	c = painterly_albedo(c, world_pos);
	float up = clamp(normal.y, 0.0, 1.0);
	// The bed dip is shared at lattice corners, so banks blend continuously across chunk edges.
	float shore = smoothstep(0.01, 0.32, UV.x) * (1.0 - smoothstep(0.5, 0.85, UV.x));
	if (shore > 0.001) {
		c = mix(c, shore_colour * material_detail(world_pos.xz * 0.32, 7.0), shore * 0.65);
	}
	// Steep slopes show bare rock instead of grass.
	float cliff = 1.0 - smoothstep(0.62, 0.8, up);
	if (cliff > 0.001) {
		c = mix(c, painterly_albedo(cliff_colour, world_pos) * material_triplanar(world_pos * 0.4, normal, 11.0), cliff);
	}
	float cover = snow * smoothstep(0.55, 0.9, up) * smoothstep(0.25, 0.55, n + snow * 0.4);
	vec3 snow_detail = mix(material_detail(world_pos.xz * 0.25, 8.0), material_detail(world_pos.xz * 0.25, 9.0), vnoise(world_pos.xz * 0.17));
	c = mix(c, vec3(0.80, 0.85, 0.92) * snow_detail, cover);
	vec2 uv = world_pos.xz / map_size;
	if (overlay_on > 0.5 && uv.x >= 0.0 && uv.y >= 0.0 && uv.x <= 1.0 && uv.y <= 1.0) {
		vec4 o = texture(overlay_tex, uv);
		c = mix(c, srgb_to_linear(o.rgb), o.a);
		float fr = texture(frame_tex, uv).r;
		vec2 f = fract(world_pos.xz);
		float edge = min(min(f.x, f.y), min(1.0 - f.x, 1.0 - f.y));
		float line = fr * (1.0 - smoothstep(0.035, 0.07, edge));
		c = mix(c, mix(vec3(1.0), o.rgb * 1.4, 0.35), line);
	}
	if (grid_on > 0.5) {
		vec2 g = fract(world_pos.xz);
		float ge = min(min(g.x, g.y), min(1.0 - g.x, 1.0 - g.y));
		c *= 1.0 - 0.18 * (1.0 - smoothstep(0.01, 0.03, ge));
	}
	ALBEDO = c;
	ROUGHNESS = 0.95;
}

void light() {
	DIFFUSE_LIGHT += painterly_diffuse(NORMAL, LIGHT, VIEW, ALBEDO, LIGHT_COLOR, ATTENUATION);
}

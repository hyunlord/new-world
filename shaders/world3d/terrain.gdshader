// terrain.gdshader — the 3D view's ground (scripts/world3d/terrain3d.gd, M3D-1 scope 3/8/9).
// Albedo is the per-corner vertex colour (terrain code colour, clearing, floors, fields) times the season tint,
// broken up by a fine value noise; slopes steeper than ~40° turn to `cliff_colour` rock; snow covers upward faces by `snow` (winter). `overlay_tex` is one texel per map
// tile (RGBA fill; alpha 0 = nothing) for the overlays and order feedback; `frame_tex` marks tiles whose outline is
// drawn (R = outline strength) so hover, drag rectangles and refused tiles read as framed tiles.
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;
#include "res://shaders/world3d/material_texture.gdshaderinc"
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
// M-GROW 3.2: worn path stages (0 grass, 1 trodden, 2 dirt, 3 paved).
// Stage colours are multiplied by the art-tex atlas detail: packed_earth (tiles 4/5) for trodden/dirt, paved_road (23).
uniform sampler2D path_tex : filter_linear, repeat_disable;
uniform float ground_scale = 0.68;
uniform float edge_noise_scale = 1.65;
uniform float edge_warp = 0.32;
uniform float macro_scale = 0.095;
uniform float macro_strength = 0.22;
uniform float normal_strength = 0.12;
uniform float ambient_occlusion = 0.22;
uniform vec2 roughness_range = vec2(0.79, 0.99);
uniform vec3 path_strength = vec3(0.52, 0.82, 0.94);
uniform vec3 trodden_colour : source_color = vec3(0.42, 0.38, 0.28);
uniform vec3 dirt_colour : source_color = vec3(0.50, 0.40, 0.25);
uniform vec3 paved_colour : source_color = vec3(0.55, 0.50, 0.42);

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

vec2 biome_at(vec2 tile_pos) {
	return texture(biome_tex, (tile_pos + 0.5) / map_size).rg;
}

vec3 ground_detail(vec2 biome, vec2 tex_pos, float variant) {
	float tile = floor(biome.r * 255.0 + 0.5);
	if (tile < 0.5) {
		tile = autumn > 0.5 ? 2.0 : variant;
	} else if (abs(tile - 4.0) < 0.5) {
		tile += variant;
	}
	vec3 detail = material_detail(tex_pos, tile);
	return biome.g > 0.001 ? mix(detail, material_detail(tex_pos, 4.0 + variant), biome.g) : detail;
}

vec3 blended_ground(vec2 p, vec2 tex_pos, float variant) {
	vec2 base = floor(p - 0.5);
	vec2 f = smoothstep(vec2(0.0), vec2(1.0), fract(p - 0.5));
	vec2 a = biome_at(base);
	vec2 b = biome_at(base + vec2(1.0, 0.0));
	vec2 c = biome_at(base + vec2(0.0, 1.0));
	vec2 d = biome_at(base + vec2(1.0));
	vec3 da = ground_detail(a, tex_pos, variant);
	// Most pixels are inside a single biome. Avoid resampling its atlas cell.
	vec3 db = a == b ? da : ground_detail(b, tex_pos, variant);
	vec3 dc = a == c ? da : ground_detail(c, tex_pos, variant);
	vec3 dd = a == d ? da : ground_detail(d, tex_pos, variant);
	return mix(mix(da, db, f.x), mix(dc, dd, f.x), f.y);
}

void fragment() {
	vec2 p = world_pos.xz;
	vec2 tex_pos = p * ground_scale;
	float n = vnoise(p * edge_noise_scale);
	float macro_noise = vnoise(p * macro_scale);
	vec2 warp = vec2(n - 0.5, vnoise(p.yx * edge_noise_scale + 3.0) - 0.5) * edge_warp;
	float variant = step(0.5, vnoise(p * macro_scale + 7.0));
	vec3 detail = blended_ground(p + warp, tex_pos, variant);
	vec3 c = srgb_to_linear(COLOR.rgb) * season_tint * detail * (1.0 + macro_strength * (macro_noise - 0.5));
	// Smooth coverage across the original 0/1/2/3 worn-path bytes without changing their storage or priority.
	float stage = texture(path_tex, (p + warp) / map_size).r * 255.0;
	if (stage > 0.03) {
		vec3 earth = material_detail(tex_pos, 4.0 + variant);
		float worn = smoothstep(0.03, 1.0, stage);
		float dirt = smoothstep(1.0, 2.0, stage);
		float paved = smoothstep(2.0, 3.0, stage);
		c = mix(c, trodden_colour * earth, worn * path_strength.x);
		c = mix(c, dirt_colour * earth, dirt * path_strength.y);
		if (paved > 0.0) {
			c = mix(c, paved_colour * material_detail(tex_pos, 23.0), paved * path_strength.z);
		}
	}
	float up = clamp(world_normal.y, 0.0, 1.0);
	float rock = 1.0 - smoothstep(0.62, 0.8, up);
	if (rock > 0.0) {
		c = mix(c, cliff_colour * (0.85 + 0.3 * n) * material_triplanar(world_pos * ground_scale, world_normal, 11.0), rock);
	}
	if (snow > 0.0) {
		float cover = snow * smoothstep(0.55, 0.9, up) * smoothstep(0.25, 0.55, n + snow * 0.4);
		vec3 snow_detail = mix(material_detail(tex_pos, 8.0), material_detail(tex_pos, 9.0), macro_noise);
		c = mix(c, vec3(0.80, 0.85, 0.92) * snow_detail, cover);
	}

	// Recover the detail height gradient in world X/Z to leave the terrain geometry untouched.
	vec2 dx = dFdx(p);
	vec2 dy = dFdy(p);
	float h = detail.r;
	float det = dx.x * dy.y - dx.y * dy.x;
	if (abs(det) > 0.000001) {
		vec2 gradient = (vec2(dy.y, -dy.x) * dFdx(h) + vec2(-dx.y, dx.x) * dFdy(h)) / det;
		vec3 nrm = normalize(world_normal - normal_strength * vec3(clamp(gradient.x, -1.0, 1.0), 0.0, clamp(gradient.y, -1.0, 1.0)) * (1.0 - rock));
		NORMAL = normalize((VIEW_MATRIX * vec4(nrm, 0.0)).xyz);
	}
	AO = 1.0 - ambient_occlusion * clamp((1.0 - h) + 0.35 * rock, 0.0, 1.0);
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
	ROUGHNESS = mix(roughness_range.x, roughness_range.y, clamp(h, 0.0, 1.0));
}

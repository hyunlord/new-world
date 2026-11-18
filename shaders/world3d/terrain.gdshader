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
uniform sampler2D path_tex : filter_nearest, repeat_disable;
uniform vec3 trodden_colour : source_color = vec3(0.66, 0.60, 0.38);
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

vec3 ground_detail(vec2 tile_pos, vec2 p) {
	vec2 biome = texture(biome_tex, (tile_pos + 0.5) / map_size).rg;
	float tile = floor(biome.r * 255.0 + 0.5);
	float variant = step(0.5, hash12(tile_pos));
	if (tile < 0.5) {
		tile = autumn > 0.5 ? 2.0 : variant;
	} else if (abs(tile - 4.0) < 0.5) {
		tile += variant;
	}
	vec3 detail = material_detail(p * 0.32, tile);
	return biome.g > 0.001 ? mix(detail, material_detail(p * 0.32, 4.0 + variant), biome.g) : detail;
}

vec3 blended_ground(vec2 p) {
	// The same four neighbouring tiles that supply each vertex's existing colour.
	vec2 base = floor(p - 0.5);
	vec2 f = fract(p - 0.5);
	return mix(mix(ground_detail(base, p), ground_detail(base + vec2(1.0, 0.0), p), f.x),
		mix(ground_detail(base + vec2(0.0, 1.0), p), ground_detail(base + vec2(1.0), p), f.x), f.y);
}

void fragment() {
	vec3 c = srgb_to_linear(COLOR.rgb) * season_tint;
	c *= blended_ground(world_pos.xz);
	float n = vnoise(world_pos.xz * 3.0) * 0.6 + vnoise(world_pos.xz * 11.0) * 0.4;
	c *= 0.90 + 0.20 * n;
	// M-GROW 3.2: blend path stage colour onto the ground.
	vec2 puv = world_pos.xz / map_size;
	if (puv.x >= 0.0 && puv.y >= 0.0 && puv.x <= 1.0 && puv.y <= 1.0) {
		// Worn stages spread 0.35 tile past their tile (max of the neighbours) so a one-tile path reads as a band;
		// the spread edge blends weaker than the tile itself.
		float own = texture(path_tex, puv).r * 255.0;
		vec2 o = vec2(0.35) / map_size;
		float spread = max(max(texture(path_tex, puv + vec2(o.x, 0.0)).r, texture(path_tex, puv - vec2(o.x, 0.0)).r),
			max(texture(path_tex, puv + vec2(0.0, o.y)).r, texture(path_tex, puv - vec2(0.0, o.y)).r)) * 255.0;
		float stage = own;
		float edge = 1.0;
		if (spread > own + 0.5 && spread < 2.5) {
			stage = spread;
			edge = 0.7;
		}
		vec3 earth = material_detail(world_pos.xz * 0.32, 4.0 + step(0.5, hash12(floor(world_pos.xz))));
		if (stage > 0.5 && stage < 1.5) {
			// Trodden: pale, dry, flattened grass — lighter and yellower than any grass checker tone.
			c = mix(c, srgb_to_linear(trodden_colour) * earth, (0.55 + 0.1 * n) * edge);
		} else if (stage > 1.5 && stage < 2.5) {
			c = mix(c, srgb_to_linear(dirt_colour) * earth, (0.75 + 0.1 * n) * edge);
		} else if (stage > 2.5) {
			c = mix(c, srgb_to_linear(paved_colour) * material_detail(world_pos.xz * 0.32, 23.0), 0.75);
		}
	}
	float up = clamp(world_normal.y, 0.0, 1.0);
	// Steep slopes show bare rock instead of grass.
	c = mix(c, cliff_colour * (0.85 + 0.3 * n) * material_triplanar(world_pos * 0.4, world_normal, 11.0), 1.0 - smoothstep(0.62, 0.8, up));
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

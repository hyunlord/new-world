// terrain.gdshader — the 3D view's ground (scripts/world3d/terrain3d.gd, M3D-1 scope 3/8/9).
// Albedo is the per-corner vertex colour (terrain code colour, clearing, floors, fields) times the season tint,
// broken up by a fine value noise; snow covers upward faces by `snow` (winter). `overlay_tex` is one texel per map
// tile (RGBA fill; alpha 0 = nothing) for the overlays and order feedback; `frame_tex` marks tiles whose outline is
// drawn (R = outline strength) so hover, drag rectangles and refused tiles read as framed tiles.
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;

uniform vec3 season_tint = vec3(1.0);
uniform float snow = 0.0;
uniform vec2 map_size = vec2(192.0, 192.0);
uniform sampler2D overlay_tex : filter_nearest, repeat_disable, source_color;
uniform sampler2D frame_tex : filter_nearest, repeat_disable;
uniform float overlay_on = 0.0;
uniform float grid_on = 0.0;

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

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	vec3 c = COLOR.rgb * season_tint;
	float n = vnoise(world_pos.xz * 3.0) * 0.6 + vnoise(world_pos.xz * 11.0) * 0.4;
	c *= 0.90 + 0.20 * n;
	float up = clamp(world_normal.y, 0.0, 1.0);
	float cover = snow * smoothstep(0.55, 0.9, up) * smoothstep(0.25, 0.55, n + snow * 0.4);
	c = mix(c, vec3(0.90, 0.93, 0.97), cover);
	vec2 uv = world_pos.xz / map_size;
	if (overlay_on > 0.5 && uv.x >= 0.0 && uv.y >= 0.0 && uv.x <= 1.0 && uv.y <= 1.0) {
		vec4 o = texture(overlay_tex, uv);
		c = mix(c, o.rgb, o.a);
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

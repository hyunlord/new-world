// flora.gdshader — trees, bushes, rocks and outcrops of the 3D view (scripts/world3d/flora3d.gd, M3D-1 scope 4).
// albedo × the MultiMesh instance colour (per-plant value variation); `sway` bends vertices above 1 m with a slow
// wind; `snow` whitens upward faces (winter); `glow` makes a surface emissive (embers); `vein` adds a faint
// self-light and a sparkle so ore veins read at the default zoom.
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;

uniform vec3 albedo : source_color = vec3(0.4, 0.55, 0.25);
uniform float sway = 0.0;
uniform float snow = 0.0;
uniform float glow = 0.0;
uniform float vein = 0.0;

varying vec3 world_normal;
varying vec3 world_pos;

void vertex() {
	vec3 origin = (MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
	float phase = origin.x * 0.37 + origin.z * 0.61;
	float bend = max(VERTEX.y - 1.0, 0.0) * sway;
	VERTEX.x += sin(TIME * 1.3 + phase) * bend * 0.025;
	VERTEX.z += cos(TIME * 1.1 + phase * 1.3) * bend * 0.018;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec3 c = albedo * COLOR.rgb;
	float up = clamp(world_normal.y, 0.0, 1.0);
	c = mix(c, vec3(0.92, 0.94, 0.98), snow * smoothstep(0.45, 0.85, up));
	ALBEDO = c;
	ROUGHNESS = 0.9;
	if (glow > 0.5) {
		EMISSION = albedo * 2.5;
	}
	if (vein > 0.5) {
		float sparkle = step(0.93, fract(sin(dot(floor(world_pos * 9.0), vec3(12.9898, 78.233, 37.719))) * 43758.5453));
		EMISSION = albedo * (0.35 + sparkle * 1.5);
		ROUGHNESS = 0.35;
		METALLIC = 0.3;
	}
}

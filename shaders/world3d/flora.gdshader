// flora.gdshader — trees, bushes, rocks and outcrops of the 3D view (scripts/world3d/flora3d.gd, M3D-1 scope 4).
// One merged surface per model variant (flora3d.gd `_mesh_variant`): COLOR is the baked surface colour (sRGB) × the
// MultiMesh instance colour (per-plant value variation); UV.y is the sway (bends vertices above 1 m with a slow wind),
// UV.x the kind (1 glow: emissive embers; 2 vein: a faint self-light and a sparkle so ore veins read at the default
// zoom); the global `w3d_snow` whitens upward faces (winter).
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;

global uniform float w3d_snow;
varying float kind;
// Set every frame by camera_rig.gd: the camera, the look target and the x-ray radius (metres).
global uniform vec3 w3d_camera;
global uniform vec3 w3d_focus;
global uniform float w3d_xray;
// The per-tile overlay (overlay3d.gd): a plant on a framed tile (marked, dragged over) takes its colour.
global uniform sampler2D w3d_overlay : filter_nearest, repeat_disable;
global uniform sampler2D w3d_frame : filter_nearest, repeat_disable;
global uniform vec2 w3d_map_size;
varying vec4 mark;

varying vec3 world_normal;
varying vec3 world_pos;

void vertex() {
	vec3 origin = (MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
	float phase = origin.x * 0.37 + origin.z * 0.61;
	float bend = max(VERTEX.y - 1.0, 0.0) * UV.y;
	kind = UV.x;
	VERTEX.x += sin(TIME * 1.3 + phase) * bend * 0.025;
	VERTEX.z += cos(TIME * 1.1 + phase * 1.3) * bend * 0.018;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	vec2 uv = (floor(origin.xz) + 0.5) / max(w3d_map_size, vec2(1.0));
	mark = vec4(textureLod(w3d_overlay, uv, 0.0).rgb, textureLod(w3d_frame, uv, 0.0).r * textureLod(w3d_overlay, uv, 0.0).a);
}

vec3 srgb_to_linear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

float bayer4(vec2 p) {
	ivec2 q = ivec2(mod(p, 4.0));
	int idx = q.x + q.y * 4;
	float m[16] = float[](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
	return (m[idx] + 0.5) / 16.0;
}

void fragment() {
	// X-ray: plants between the camera and the look target dissolve inside a cylinder around that line.
	vec3 axis = w3d_focus - w3d_camera;
	float len = length(axis);
	if (w3d_xray > 0.0 && len > 0.01) {
		vec3 dir = axis / len;
		float t = dot(world_pos - w3d_camera, dir);
		if (t < len - 0.8) {
			float d = length(world_pos - (w3d_camera + dir * t));
			float keep = smoothstep(w3d_xray * 0.7, w3d_xray * 1.15, d);
			if (bayer4(FRAGCOORD.xy) > keep) {
				discard;
			}
		}
	}
	vec3 c = srgb_to_linear(COLOR.rgb);
	if (mark.a > 0.01) {
		c = mix(c, mark.rgb * mark.rgb, clamp(mark.a * 1.6, 0.0, 0.75));
	}
	float up = clamp(world_normal.y, 0.0, 1.0);
	c = mix(c, vec3(0.80, 0.85, 0.92), w3d_snow * smoothstep(0.45, 0.85, up));
	ALBEDO = c;
	ROUGHNESS = 0.9;
	if (kind > 0.5 && kind < 1.5) {
		EMISSION = c * 2.5;
	}
	if (kind > 1.5) {
		float sparkle = step(0.93, fract(sin(dot(floor(world_pos * 9.0), vec3(12.9898, 78.233, 37.719))) * 43758.5453));
		EMISSION = c * (0.35 + sparkle * 1.5);
		ROUGHNESS = 0.35;
		METALLIC = 0.3;
	}
}

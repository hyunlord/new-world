shader_type sky;

uniform vec3 sky_top : source_color;
uniform vec3 sky_horizon : source_color;
uniform vec3 ground_horizon : source_color;
uniform vec3 ground_bottom : source_color;
uniform vec3 cloud_colour : source_color;
uniform float cloud_scale;
uniform float cloud_opacity;
uniform vec2 cloud_thresholds;

float cloud_hash(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float cloud_noise(vec2 p) {
	vec2 cell = floor(p);
	vec2 t = fract(p);
	t = t * t * (3.0 - 2.0 * t);
	return mix(
		mix(cloud_hash(cell), cloud_hash(cell + vec2(1.0, 0.0)), t.x),
		mix(cloud_hash(cell + vec2(0.0, 1.0)), cloud_hash(cell + vec2(1.0)), t.x),
		t.y
	);
}

void sky() {
	float elevation = EYEDIR.y;
	if (elevation < 0.0) {
		COLOR = mix(ground_horizon, ground_bottom, smoothstep(0.0, 1.0, -elevation));
	} else {
		vec2 p = EYEDIR.xz * cloud_scale / (elevation + 0.18);
		float cloud = cloud_noise(p) * 0.63 + cloud_noise(p * 2.13) * 0.27 + cloud_noise(p * 4.51) * 0.10;
		float mask = smoothstep(cloud_thresholds.x, cloud_thresholds.y, cloud);
		float above_horizon = smoothstep(0.01, 0.19, elevation);
		vec3 clear_sky = mix(sky_horizon, sky_top, sqrt(elevation));
		COLOR = mix(clear_sky, cloud_colour, mask * cloud_opacity * above_horizon);
	}
}

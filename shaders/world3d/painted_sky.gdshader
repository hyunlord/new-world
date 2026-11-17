// Painted upper hemisphere; existing light, ambient colour and fog stay clock-driven.
shader_type sky;

uniform sampler2D day_sky : source_color, filter_linear_mipmap, repeat_disable;
uniform sampler2D dusk_sky : source_color, filter_linear_mipmap, repeat_disable;
uniform sampler2D night_sky : source_color, filter_linear_mipmap, repeat_disable;
uniform sampler2D winter_sky : source_color, filter_linear_mipmap, repeat_disable;
uniform sampler2D day_cloud : source_color, filter_linear_mipmap, repeat_disable;
uniform sampler2D dusk_cloud : source_color, filter_linear_mipmap, repeat_disable;
uniform sampler2D night_cloud : source_color, filter_linear_mipmap, repeat_disable;
uniform sampler2D winter_cloud : source_color, filter_linear_mipmap, repeat_disable;
uniform sampler2D hills : source_color, filter_linear_mipmap, repeat_disable;
uniform vec3 blend_weights = vec3(0.0);
uniform float cloud_opacity = 0.65;
uniform float hills_opacity = 0.5;
uniform vec3 ground_colour : source_color = vec3(0.3, 0.32, 0.3);

vec4 premultiply(vec4 c) {
	return vec4(c.rgb * c.a, c.a);
}

void sky() {
	// Mirrored longitude closes the panorama without altering generated cloud alpha.
	vec2 uv = vec2(abs(SKY_COORDS.x * 2.0 - 1.0), clamp(SKY_COORDS.y * 2.0, 0.0, 1.0));
	vec3 base = mix(texture(day_sky, uv).rgb, texture(winter_sky, uv).rgb, blend_weights.z);
	base = mix(base, texture(dusk_sky, uv).rgb, blend_weights.y);
	base = mix(base, texture(night_sky, uv).rgb, blend_weights.x);
	vec4 cloud = mix(premultiply(texture(day_cloud, uv)), premultiply(texture(winter_cloud, uv)), blend_weights.z);
	cloud = mix(cloud, premultiply(texture(dusk_cloud, uv)), blend_weights.y);
	cloud = mix(cloud, premultiply(texture(night_cloud, uv)), blend_weights.x);
	base = base * (1.0 - cloud.a * cloud_opacity) + cloud.rgb * cloud_opacity;
	vec4 ridge = texture(hills, uv);
	base = mix(base, ridge.rgb, ridge.a * hills_opacity);
	COLOR = mix(base, ground_colour, smoothstep(0.0, 0.4, -EYEDIR.y));
}

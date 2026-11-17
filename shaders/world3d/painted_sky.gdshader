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

vec4 panorama(sampler2D tex, vec2 uv) {
	// Overlap the ends instead of mirroring the whole cloud silhouette.
	uv.x *= 0.85;
	vec4 primary = premultiply(texture(tex, uv));
	if (uv.x < 0.15) {
		vec4 wrapped = premultiply(texture(tex, vec2(uv.x + 0.85, uv.y)));
		return mix(wrapped, primary, smoothstep(0.0, 0.15, uv.x));
	}
	return primary;
}

void sky() {
	vec2 uv = vec2(SKY_COORDS.x, clamp(SKY_COORDS.y * 2.0, 0.0, 1.0));
	vec3 base = mix(panorama(day_sky, uv).rgb, panorama(winter_sky, uv).rgb, blend_weights.z);
	base = mix(base, panorama(dusk_sky, uv).rgb, blend_weights.y);
	base = mix(base, panorama(night_sky, uv).rgb, blend_weights.x);
	vec4 cloud = mix(panorama(day_cloud, uv), panorama(winter_cloud, uv), blend_weights.z);
	cloud = mix(cloud, panorama(dusk_cloud, uv), blend_weights.y);
	cloud = mix(cloud, panorama(night_cloud, uv), blend_weights.x);
	base = base * (1.0 - cloud.a * cloud_opacity) + cloud.rgb * cloud_opacity;
	vec4 ridge = panorama(hills, uv);
	base = base * (1.0 - ridge.a * hills_opacity) + ridge.rgb * hills_opacity;
	COLOR = mix(base, ground_colour, smoothstep(0.0, 0.4, -EYEDIR.y));
}

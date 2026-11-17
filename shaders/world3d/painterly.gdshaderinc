// Shared opaque-world response: broad pigment variation, wrapped light and a restrained lit rim.
float painted_noise(vec3 p) {
	vec3 i = floor(p);
	vec3 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	vec3 q = i + f;
	return 0.5 + 0.25 * sin(q.x * 1.73 + q.y * 2.31 + q.z * 0.83)
		+ 0.25 * sin(q.x * 0.61 - q.y * 1.13 + q.z * 1.57);
}

vec3 painterly_albedo(vec3 colour, vec3 position) {
	float pigment = painted_noise(position * 1.4);
	float wash = painted_noise(position * 0.19);
	return colour * (0.92 + 0.12 * pigment + 0.08 * wash);
}

vec3 painterly_diffuse(vec3 normal, vec3 light, vec3 view, vec3 pigment_colour, vec3 light_colour, float attenuation) {
	float half_lambert = clamp(dot(normal, light) * 0.5 + 0.5, 0.0, 1.0);
	float ramp = smoothstep(0.14, 0.98, half_lambert * half_lambert);
	float rim = pow(1.0 - max(dot(normal, view), 0.0), 3.0) * ramp * 0.045;
	// Godot applies ALBEDO after light(); keep pigment out of the diffuse term to avoid squaring it.
	vec3 rim_tint = mix(vec3(1.0), clamp(pigment_colour + vec3(0.5), vec3(0.0), vec3(1.0)), 0.15);
	return (ramp + rim * rim_tint) * light_colour * attenuation / PI;
}

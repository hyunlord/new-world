// One atlas sampler keeps existing mesh batches and material draw calls intact.
uniform sampler2D material_atlas : source_color, filter_linear_mipmap, repeat_disable;
uniform float texture_strength = 0.0;
uniform float texture_repeat_scale = 1.0;
uniform float material_luminance[64];
uniform sampler2D material_normal_atlas : hint_normal, filter_linear_mipmap, repeat_disable;

vec2 material_uv(vec2 p, float tile) {
	vec2 cell = vec2(mod(tile, 8.0), floor(tile / 8.0));
	return (cell + (vec2(8.0) + fract(p) * 1008.0) / 1024.0) / 8.0;
}

vec3 material_detail(vec2 p, float tile) {
	if (tile < -0.5 || tile > 63.0 || texture_strength < 0.001) { return vec3(1.0); }
	p *= texture_repeat_scale;
	vec2 uv = material_uv(p, tile);
	// Explicit derivatives avoid a false coarse mip at the periodic wrap seam.
	// Cap the footprint at the 8-pixel gutter so distant mips cannot read adjacent materials.
	vec2 dx = clamp(dFdx(p) * (1008.0 / 8192.0), vec2(-8.0 / 8192.0), vec2(8.0 / 8192.0));
	vec2 dy = clamp(dFdy(p) * (1008.0 / 8192.0), vec2(-8.0 / 8192.0), vec2(8.0 / 8192.0));
	vec3 texel = textureGrad(material_atlas, uv, dx, dy).rgb;
	float value = dot(texel, vec3(0.2126, 0.7152, 0.0722));
	float mean_value = material_luminance[int(tile + 0.5)];
	// Normalise around the authored palette; the floor limits amplification of dark materials.
	float detail = clamp(1.0 + (value - mean_value) / max(mean_value, 0.2), 0.0, 4.0);
	// Retain painted masses at gameplay zoom; fade strongly minified detail to the palette.
	float footprint = max(length(dFdx(p) * 1008.0), length(dFdy(p) * 1008.0));
	float resolved = 1.0 - smoothstep(16.0, 32.0, footprint);
	return vec3(mix(1.0, detail, texture_strength * resolved));
}

vec3 material_bump(vec3 p, vec3 n, float tile) {
	if (tile < -0.5 || tile > 63.0 || texture_strength < 0.001) { return n; }
	p *= texture_repeat_scale;
	vec3 dx = clamp(dFdx(p) * (1008.0 / 8192.0), vec3(-8.0 / 8192.0), vec3(8.0 / 8192.0));
	vec3 dy = clamp(dFdy(p) * (1008.0 / 8192.0), vec3(-8.0 / 8192.0), vec3(8.0 / 8192.0));
	vec2 a = textureGrad(material_normal_atlas, material_uv(p.zy, tile), dx.zy, dy.zy).rg * 2.0 - 1.0;
	vec2 b = textureGrad(material_normal_atlas, material_uv(p.xz, tile), dx.xz, dy.xz).rg * 2.0 - 1.0;
	vec2 c = textureGrad(material_normal_atlas, material_uv(p.xy, tile), dx.xy, dy.xy).rg * 2.0 - 1.0;
	vec3 w = pow(abs(n), vec3(4.0));
	w /= max(w.x + w.y + w.z, 0.0001);
	vec3 perturb = vec3(0.0, a.y, a.x) * w.x * sign(n.x) + vec3(b.x, 0.0, b.y) * w.y * sign(n.y) + vec3(c.x, c.y, 0.0) * w.z * sign(n.z);
	return normalize(n + (perturb - n * dot(perturb, n)) * 0.15 * texture_strength);
}

vec3 material_triplanar(vec3 p, vec3 normal, float tile) {
	vec3 w = pow(abs(normal), vec3(4.0));
	w /= max(w.x + w.y + w.z, 0.0001);
	return material_detail(p.zy, tile) * w.x + material_detail(p.xz, tile) * w.y + material_detail(p.xy, tile) * w.z;
}

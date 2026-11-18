// Slice 1 CC0 surfaces. Texture-array layers match slice_pbr.gd.
uniform sampler2DArray slice_albedo : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2DArray slice_normal : filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2DArray slice_roughness : filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2DArray slice_ao : filter_linear_mipmap_anisotropic, repeat_enable;

const float SLICE_GRASS = 0.0;
const float SLICE_FOREST = 1.0;
const float SLICE_DIRT = 2.0;
const float SLICE_MUD = 3.0;
const float SLICE_ROCK = 4.0;
const float SLICE_OAK = 5.0;
const float SLICE_PINE = 6.0;
const float SLICE_LOG = 7.0;
const float SLICE_PLANK = 8.0;
const float SLICE_THATCH = 9.0;

struct SliceSurface {
	vec3 albedo;
	vec3 bump;
	float roughness;
	float ao;
};

float slice_slot(float slot) {
	int tile = int(floor(slot + 0.5));
	if (tile == 4 || tile == 5) { return SLICE_DIRT; }
	if (tile == 6 || tile == 19 || tile == 20) { return SLICE_MUD; }
	if (tile == 11 || tile == 12 || tile == 21 || tile == 22 || tile == 23) { return SLICE_ROCK; }
	if (tile == 14) { return SLICE_OAK; }
	if (tile == 15 || tile == 26) { return SLICE_PINE; }
	if (tile == 16) { return SLICE_LOG; }
	if (tile == 17 || tile == 18) { return SLICE_PLANK; }
	if (tile == 24 || tile == 25) { return SLICE_THATCH; }
	return -1.0;
}

SliceSurface slice_mix(SliceSurface a, SliceSurface b, float weight) {
	return SliceSurface(mix(a.albedo, b.albedo, weight), mix(a.bump, b.bump, weight),
		mix(a.roughness, b.roughness, weight), mix(a.ao, b.ao, weight));
}

SliceSurface slice_sample(vec2 p, vec2 dx, vec2 dy, float layer) {
	vec3 uv = vec3(p, layer);
	return SliceSurface(
		textureGrad(slice_albedo, uv, dx, dy).rgb,
		textureGrad(slice_normal, uv, dx, dy).rgb * 2.0 - 1.0,
		textureGrad(slice_roughness, uv, dx, dy).r,
		textureGrad(slice_ao, uv, dx, dy).r
	);
}

SliceSurface slice_triplanar(vec3 p, vec3 dx, vec3 dy, vec3 n, float layer) {
	vec3 weights = pow(abs(n), vec3(4.0));
	weights /= max(dot(weights, vec3(1.0)), 0.0001);
	// Keep axis-facing projections right handed, including the opposite sides of logs and rocks.
	vec3 hand = mix(vec3(-1.0), vec3(1.0), step(vec3(0.0), n));
	vec2 sx = vec2(-hand.x, 1.0);
	vec2 sy = vec2(1.0, -hand.y);
	vec2 sz = vec2(hand.z, 1.0);
	SliceSurface a = slice_sample(p.zy * sx, dx.zy * sx, dy.zy * sx, layer);
	SliceSurface b = slice_sample(p.xz * sy, dx.xz * sy, dy.xz * sy, layer);
	SliceSurface c = slice_sample(p.xy * sz, dx.xy * sz, dy.xy * sz, layer);
	a.bump = vec3(0.0, a.bump.y, a.bump.x * sx.x);
	b.bump = vec3(b.bump.x, 0.0, b.bump.y * sy.y);
	c.bump = vec3(c.bump.x * sz.x, c.bump.y, 0.0);
	return SliceSurface(
		a.albedo * weights.x + b.albedo * weights.y + c.albedo * weights.z,
		a.bump * weights.x + b.bump * weights.y + c.bump * weights.z,
		dot(vec3(a.roughness, b.roughness, c.roughness), weights),
		dot(vec3(a.ao, b.ao, c.ao), weights)
	);
}

vec3 slice_perturb(vec3 n, vec3 bump, float strength) {
	return normalize(n + strength * (bump - n * dot(bump, n)));
}

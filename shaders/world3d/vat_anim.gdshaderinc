// vat_anim.gdshaderinc — the shared vertex animation of the VAT crowds (vat.gdshader, vat_outline.gdshader).
// `vat_position` reads the vertex's position for the current frame (column = UV2.x · width; rows: the clip's frames from
// INSTANCE_CUSTOM.r = start × 256 + count, phase INSTANCE_CUSTOM.g), blending two frames on World3D's clock `w3d_time`.
// `far_scale_at` enlarges a far figure (beyond far_scale.x metres, up to far_scale.y) so crowds stay readable at ¼×.
global uniform float w3d_time;
uniform sampler2D vat : filter_nearest, repeat_disable;
uniform float width = 1.0;
uniform float fps = 15.0;
uniform vec2 far_scale = vec2(1.0e6, 1.0);

vec3 vat_position(vec2 uv2, vec4 custom) {
	int column = int(uv2.x * width);
	float pack = floor(custom.r + 0.5);
	float start = floor(pack / 256.0);
	float count = max(1.0, mod(pack, 256.0));
	float f = mod((w3d_time + custom.g) * fps, count);
	int f0 = int(floor(f));
	int f1 = int(mod(float(f0 + 1), count));
	vec3 p0 = texelFetch(vat, ivec2(column, int(start) + f0), 0).xyz;
	vec3 p1 = texelFetch(vat, ivec2(column, int(start) + f1), 0).xyz;
	return mix(p0, p1, fract(f));
}

float far_scale_at(float dist) {
	return clamp(dist / far_scale.x, 1.0, far_scale.y);
}

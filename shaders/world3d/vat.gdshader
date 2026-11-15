// vat.gdshader — vertex-animation-texture crowds (scripts/world3d/vat3d.gd; people3d.gd, animals3d.gd).
// Each vertex reads its position for the current frame from `vat` (column = UV2.x · width, the unsplit vertex; rows:
// the clip's frames), blending two frames; the fragment derives the flat face normal from screen derivatives. Tint by region (UV2.y): 1 → instance COLOR,
// 2 → INSTANCE_CUSTOM.b, 3 → INSTANCE_CUSTOM.a (packed r·65536 + g·256 + b), 0 → `other_colour`. `textured`
// characters sample `atlas` at UV instead: regions 1–3 take the person's tint shaded by the texel's luminance over the
// region's mean luminance (painted detail stays, the hue is the person's); region 0 is drawn as painted.
// INSTANCE_CUSTOM.r = clip start × 256 + clip frame count, .g = phase in seconds (per instance, so a crowd is not in step).
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;

uniform sampler2D vat : filter_nearest, repeat_disable;
uniform float width = 1.0;
uniform float fps = 15.0;
uniform vec3 other_colour : source_color = vec3(0.28, 0.2, 0.14);
uniform bool textured = false;
uniform sampler2D atlas : source_color, filter_linear_mipmap, repeat_disable;
uniform vec3 mean_skin = vec3(1.0);
uniform vec3 mean_hair = vec3(1.0);
uniform vec3 mean_cloth = vec3(1.0);

varying vec3 tint;
varying float mean_luma;

vec3 unpack_rgb(float v) {
	float x = floor(v + 0.5);
	return vec3(floor(x / 65536.0), mod(floor(x / 256.0), 256.0), mod(x, 256.0)) / 255.0;
}

float luma(vec3 c) {
	return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

vec3 srgb_to_linear(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

void vertex() {
	int column = int(UV2.x * width);
	float pack = floor(INSTANCE_CUSTOM.r + 0.5);
	float start = floor(pack / 256.0);
	float count = max(1.0, mod(pack, 256.0));
	float f = mod((TIME + INSTANCE_CUSTOM.g) * fps, count);
	int f0 = int(floor(f));
	int f1 = int(mod(float(f0 + 1), count));
	float a = fract(f);
	vec3 p0 = texelFetch(vat, ivec2(column, int(start) + f0), 0).xyz;
	vec3 p1 = texelFetch(vat, ivec2(column, int(start) + f1), 0).xyz;
	VERTEX = mix(p0, p1, a);
	int region = int(UV2.y + 0.5);
	if (region == 1) {
		tint = COLOR.rgb;
	} else if (region == 2) {
		tint = unpack_rgb(INSTANCE_CUSTOM.b);
	} else if (region == 3) {
		tint = unpack_rgb(INSTANCE_CUSTOM.a);
	} else {
		tint = other_colour;
	}
	if (textured) {
		vec3 mean = region == 1 ? mean_skin : (region == 2 ? mean_hair : mean_cloth);
		mean_luma = region == 0 ? 0.0 : max(luma(srgb_to_linear(mean)), 0.005);
		tint = srgb_to_linear(tint);
	}
}

void fragment() {
	vec3 n = normalize(cross(dFdx(VERTEX), dFdy(VERTEX)));
	NORMAL = dot(n, -VERTEX) < 0.0 ? -n : n;
	vec3 texel = textured ? texture(atlas, UV).rgb : vec3(0.0);
	ALBEDO = !textured ? srgb_to_linear(tint) : (mean_luma <= 0.0 ? texel : clamp(tint * (luma(texel) / mean_luma), 0.0, 1.0));
	ROUGHNESS = 0.85;
}

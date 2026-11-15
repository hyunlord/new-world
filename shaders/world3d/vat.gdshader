// vat.gdshader — vertex-animation-texture crowds (scripts/world3d/vat3d.gd; people3d.gd, animals3d.gd).
// Each vertex reads its position for the current frame from `vat` (column = UV2.x · width, the unsplit vertex; rows:
// the clip's frames), blending two frames; the fragment derives the flat face normal from screen derivatives. Tint by region (UV2.y): 1 → instance COLOR,
// 2 → INSTANCE_CUSTOM.b, 3 → INSTANCE_CUSTOM.a (packed r·65536 + g·256 + b), 0 → `other_colour`. `textured`
// characters sample `atlas` at UV instead and keep its painted colours as the base: skin (1) only changes value toward
// the person's skin tone (clamped to `skin_value`), hair (2) blends `hair_mix` toward the person's hair colour, and
// clothing (3) takes a small hue shift (`cloth_hue_mix`) and a clamped value shift (`cloth_value`); region 0 as painted.
// INSTANCE_CUSTOM.r = clip start × 256 + clip frame count, .g = phase in seconds (per instance, so a crowd is not in step).
shader_type spatial;
render_mode diffuse_lambert, specular_disabled;

global uniform float w3d_time;
uniform sampler2D vat : filter_nearest, repeat_disable;
uniform float width = 1.0;
uniform float fps = 15.0;
uniform vec3 other_colour : source_color = vec3(0.28, 0.2, 0.14);
uniform bool textured = false;
uniform sampler2D atlas : source_color, filter_linear_mipmap, repeat_disable;
uniform vec3 mean_skin = vec3(1.0);
uniform vec3 mean_hair = vec3(1.0);
uniform vec3 mean_cloth = vec3(1.0);
uniform vec2 skin_value = vec2(0.8, 1.2);
uniform float hair_mix = 0.6;
uniform float cloth_hue_mix = 0.3;
uniform vec2 cloth_value = vec2(0.8, 1.2);

varying vec3 tint;
varying float mean_luma;
varying flat int tint_region;

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
	float f = mod((w3d_time + INSTANCE_CUSTOM.g) * fps, count);
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
		tint_region = region;
	}
}

void fragment() {
	vec3 n = normalize(cross(dFdx(VERTEX), dFdy(VERTEX)));
	NORMAL = dot(n, -VERTEX) < 0.0 ? -n : n;
	if (!textured) {
		ALBEDO = srgb_to_linear(tint);
	} else {
		vec3 texel = texture(atlas, UV).rgb;
		vec3 c = texel;
		if (tint_region > 0) {
			// The person's colour shaded like the painted texel; `value` is how much lighter or darker they are.
			vec3 shaded = tint * (luma(texel) / mean_luma);
			float value = luma(tint) / mean_luma;
			if (tint_region == 1) {
				c = texel * clamp(value, skin_value.x, skin_value.y);
			} else if (tint_region == 2) {
				c = mix(texel, shaded, hair_mix);
			} else {
				c = mix(texel, shaded * (luma(texel) / max(luma(shaded), 0.005)), cloth_hue_mix) * clamp(value, cloth_value.x, cloth_value.y);
			}
		}
		ALBEDO = clamp(c, 0.0, 1.0);
	}
	ROUGHNESS = 0.85;
}

// vat_surface.gdshaderinc — vertex-animation-texture crowds (scripts/world3d/vat3d.gd; people3d.gd, animals3d.gd).
// Each vertex reads its position for the current frame (vat_anim.gdshaderinc; far figures enlarged); the fragment derives the flat face normal from screen derivatives. Tint by region (UV2.y): 1 → instance COLOR,
// 2 → INSTANCE_CUSTOM.b, 3 → INSTANCE_CUSTOM.a (packed r·65536 + g·256 + b), 0 → `other_colour`. `textured`
// characters sample `atlas` at UV instead and keep its painted colours as the base: skin (1) only changes value toward
// the person's skin tone (clamped to `skin_value`), hair (2) blends `hair_mix` toward the person's hair colour, and
// clothing (3) takes a small hue shift (`cloth_hue_mix`) and a clamped value shift (`cloth_value`); region 0 as painted.
// INSTANCE_CUSTOM.r = clip start × 256 + clip frame count, .g = phase in seconds (per instance, so a crowd is not in step).
#include "res://shaders/world3d/vat_anim.gdshaderinc"
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
// Far figures turn toward an unlit fill (rgb, strength) between far_scale.x and twice that distance: dark by day
// (contrast on grass, soil and snow alike), `far_fill_night` at night, so at ¼× a person reads like the 2D markers.
global uniform float w3d_night;
uniform vec4 far_fill = vec4(0.16, 0.13, 0.11, 0.0);
uniform vec3 far_fill_night : source_color = vec3(0.91, 0.89, 0.81);

varying vec3 tint;
varying float mean_luma;
varying flat int tint_region;
varying float far_k;

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
	float dist = length((MODEL_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).xyz - CAMERA_POSITION_WORLD);
	VERTEX = vat_position(UV2, INSTANCE_CUSTOM) * far_scale_at(dist);
	far_k = smoothstep(far_scale.x, far_scale.x * 2.0, dist) * far_fill.a;
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
	if (far_k > 0.0) {
		vec3 fill = srgb_to_linear(mix(far_fill.rgb, far_fill_night, clamp(w3d_night, 0.0, 1.0)));
		ALBEDO = mix(ALBEDO, fill * 0.35, far_k);
		EMISSION = fill * 0.65 * far_k;
	}
	ROUGHNESS = 0.85;
}

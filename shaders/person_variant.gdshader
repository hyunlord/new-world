// person_variant.gdshader — a route I frame recoloured to the person's face variant (M-ART2 §3.5, MART2_DECISIONS).
// The frame's mask texture (one per frame: R = skin, G = hair; tools/v8/art_masks.py) selects the pixels; each moves
// by the variant's CIELAB shift (assets/v8art/people/variant_transforms.json `skin_delta_lab`, `hair_delta_lab`), as
// the art line's baked frames do; pixels with L* <= 18 or alpha <= 127 (ink and fringe) never change. An outline of
// `outline_px` texels in `outline_color` (alpha 0 = none) marks the selected or drafted person. The per-instance
// modulate (night, mood) is captured in vertex(): in 4.6 COLOR at fragment() already includes TEXTURE.
shader_type canvas_item;

uniform sampler2D mask_tex : filter_nearest;
uniform bool has_mask = false;
uniform vec3 skin_delta = vec3(0.0);
uniform vec3 hair_delta = vec3(0.0);
uniform vec4 outline_color = vec4(0.0);
uniform float outline_px = 1.0;

varying vec4 v_tint;

void vertex() {
	v_tint = COLOR;
}

float lin(float c) { return c <= 0.04045 ? c / 12.92 : pow((c + 0.055) / 1.055, 2.4); }
float enc(float c) { c = clamp(c, 0.0, 1.0); return c <= 0.0031308 ? c * 12.92 : 1.055 * pow(c, 1.0 / 2.4) - 0.055; }
float f(float t) { const float d = 6.0 / 29.0; return t > d * d * d ? pow(t, 1.0 / 3.0) : t / (3.0 * d * d) + 4.0 / 29.0; }
float finv(float t) { const float d = 6.0 / 29.0; return t > d ? t * t * t : 3.0 * d * d * (t - 4.0 / 29.0); }

vec3 rgb_to_lab(vec3 c) {
	vec3 l = vec3(lin(c.r), lin(c.g), lin(c.b));
	float x = (0.4124564 * l.r + 0.3575761 * l.g + 0.1804375 * l.b) / 0.95047;
	float y = 0.2126729 * l.r + 0.7151522 * l.g + 0.0721750 * l.b;
	float z = (0.0193339 * l.r + 0.1191920 * l.g + 0.9503041 * l.b) / 1.08883;
	return vec3(116.0 * f(y) - 16.0, 500.0 * (f(x) - f(y)), 200.0 * (f(y) - f(z)));
}

vec3 lab_to_rgb(vec3 lab) {
	float fy = (lab.x + 16.0) / 116.0;
	float x = finv(fy + lab.y / 500.0) * 0.95047;
	float y = finv(fy);
	float z = finv(fy - lab.z / 200.0) * 1.08883;
	vec3 l = vec3(3.2404542 * x - 1.5371385 * y - 0.4985314 * z,
		-0.9692660 * x + 1.8760108 * y + 0.0415560 * z,
		0.0556434 * x - 0.2040259 * y + 1.0572252 * z);
	return vec3(enc(l.r), enc(l.g), enc(l.b));
}

void fragment() {
	vec4 c = texture(TEXTURE, UV);
	vec4 m = texture(mask_tex, UV);
	if (has_mask && c.a > 127.5 / 255.0) {
		vec3 lab = rgb_to_lab(c.rgb);
		if (lab.x > 18.0) {
			if (m.r > 0.5) {
				c.rgb = lab_to_rgb(lab + skin_delta);
			} else if (m.g > 0.5) {
				c.rgb = lab_to_rgb(lab + hair_delta);
			}
		}
	}
	if (outline_color.a > 0.0 && c.a < 0.5) {
		vec2 px = TEXTURE_PIXEL_SIZE * outline_px;
		float near = max(max(texture(TEXTURE, UV + vec2(px.x, 0.0)).a, texture(TEXTURE, UV - vec2(px.x, 0.0)).a),
			max(texture(TEXTURE, UV + vec2(0.0, px.y)).a, texture(TEXTURE, UV - vec2(0.0, px.y)).a));
		if (near >= 0.5) {
			c = outline_color;
		}
	}
	COLOR = c * v_tint;
}

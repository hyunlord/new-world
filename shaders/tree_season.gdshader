// tree_season.gdshader — a standing plant in its season (M-ART2.1 §2.2; art/flora/seasons.py `apply_tint`).
// Where the frame's canopy mask is set (and the pixel is not transparent) the RGB moves by the season's CIELAB shift
// (D65, as seasons.py and person_variant.gdshader); unmasked pixels and every alpha stay as painted. The snow cap
// (`snow_mask`: the band below each column's top canopy pixel, made by tree_layer.gd from the mask, as seasons.py
// `snow_layer`) mixes toward white by the season's snow alpha. In the first day of a season the shift and the snow
// alpha blend from the previous season's (`blend` 0 → 1); when the two seasons draw different frames (a deciduous
// tree into or out of winter) the previous frame (`alt_tex`, same canvas) dissolves into this one by a per-pixel
// hash below `blend`. Per-instance tint is captured in vertex() (a 4.6 canvas_item rule: COLOR in fragment() is
// already multiplied by TEXTURE).
shader_type canvas_item;

uniform sampler2D canopy_mask : filter_nearest;
uniform sampler2D snow_mask : filter_nearest;
uniform bool has_mask = false;
uniform bool has_snow = false;
uniform sampler2D alt_tex : filter_nearest;
uniform sampler2D alt_mask : filter_nearest;
uniform sampler2D alt_snow : filter_nearest;
uniform bool has_alt = false;
uniform bool has_alt_mask = false;
uniform bool has_alt_snow = false;
uniform vec3 delta_from = vec3(0.0);
uniform vec3 delta_to = vec3(0.0);
uniform float snow_from = 0.0;
uniform float snow_to = 0.0;
uniform float blend = 1.0;

varying vec4 v_tint;

float f_lab(float t) {
	float d = 6.0 / 29.0;
	return t > d * d * d ? pow(t, 1.0 / 3.0) : t / (3.0 * d * d) + 4.0 / 29.0;
}

float finv(float t) {
	float d = 6.0 / 29.0;
	return t > d ? t * t * t : 3.0 * d * d * (t - 4.0 / 29.0);
}

float lin(float c) {
	return c <= 0.04045 ? c / 12.92 : pow((c + 0.055) / 1.055, 2.4);
}

float enc(float c) {
	c = clamp(c, 0.0, 1.0);
	return c <= 0.0031308 ? c * 12.92 : 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

vec3 rgb_to_lab(vec3 c) {
	vec3 l = vec3(lin(c.r), lin(c.g), lin(c.b));
	float x = (0.4124564 * l.r + 0.3575761 * l.g + 0.1804375 * l.b) / 0.95047;
	float y = 0.2126729 * l.r + 0.7151522 * l.g + 0.0721750 * l.b;
	float z = (0.0193339 * l.r + 0.1191920 * l.g + 0.9503041 * l.b) / 1.08883;
	float fx = f_lab(x);
	float fy = f_lab(y);
	float fz = f_lab(z);
	return vec3(116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz));
}

vec3 lab_to_rgb(vec3 lab) {
	float fy = (lab.x + 16.0) / 116.0;
	float x = finv(fy + lab.y / 500.0) * 0.95047;
	float y = finv(fy);
	float z = finv(fy - lab.z / 200.0) * 1.08883;
	float r = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
	float g = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
	float b = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z;
	return vec3(enc(r), enc(g), enc(b));
}

float pixel_hash(vec2 p) {
	return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void vertex() {
	v_tint = COLOR;
}

void fragment() {
	vec2 px = floor(UV / TEXTURE_PIXEL_SIZE);
	bool use_alt = has_alt && pixel_hash(px) >= blend;
	vec4 c = use_alt ? texture(alt_tex, UV) : texture(TEXTURE, UV);
	bool masked = use_alt ? (has_alt_mask && texture(alt_mask, UV).r > 0.5) : (has_mask && texture(canopy_mask, UV).r > 0.5);
	vec3 delta = mix(delta_from, delta_to, blend);
	if (masked && c.a > 0.0 && dot(delta, delta) > 0.0) {
		c.rgb = lab_to_rgb(rgb_to_lab(c.rgb) + delta);
	}
	float band = use_alt ? (has_alt_snow ? texture(alt_snow, UV).r : 0.0) : (has_snow ? texture(snow_mask, UV).r : 0.0);
	float snow = band * mix(snow_from, snow_to, blend);
	c.rgb = mix(c.rgb, vec3(1.0), snow);
	COLOR = c * v_tint;
}

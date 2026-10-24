# tone.gd — pixel transforms of the M-ART2 renderer, applied once per texture at load (never per frame).
# Purpose: ports of the art line's tone code (v8/art-polish2 c8ed8608): `cap_ground` (art/compositor/tone.py: one
#   global contrast affine and one chroma factor so a ground texture's mean luminance, luminance std relative to the
#   person reference and mean saturation stay under the ENV1 locked caps), `season` (winter desaturation, drought
#   toward ochre; tone.py `season_ground`), `add_snow` (the snow overlay at ≤ the given alpha, world-tiled),
#   `lab_shift` (art/materials/tone.py `shift_image`: a D65 CIELAB shift, alpha kept) and `top_tone` (the wall top's
#   periodic blur, saturation and value). Luminance is Rec.709 on encoded sRGB, as on the art line. Every function
#   takes and returns RGBA8 Images and is deterministic (no randomness, fixed iteration counts).
# Bridge methods called: none.
extends RefCounted

const LR: float = 0.2126
const LG: float = 0.7152
const LB: float = 0.0722
## CIELAB: L* = 116 f(Y) − 16 (a colour-science constant, not a tile size).
const LAB_L_OFFSET: float = 116.0 - 100.0


## {mean, std, sat}: Rec.709 luminance mean and std and mean HSV saturation over pixels with alpha > 0
## (every `stride`-th pixel).
static func metrics(img: Image, stride: int = 1) -> Dictionary:
	var data: PackedByteArray = img.get_data()
	var n: int = 0
	var s: float = 0.0
	var s2: float = 0.0
	var sat: float = 0.0
	var step: int = 4 * maxi(1, stride)
	for i: int in range(0, data.size(), step):
		if data[i + 3] == 0:
			continue
		var r: float = data[i] / 255.0
		var g: float = data[i + 1] / 255.0
		var b: float = data[i + 2] / 255.0
		var l: float = LR * r + LG * g + LB * b
		s += l
		s2 += l * l
		var mx: float = maxf(r, maxf(g, b))
		var mn: float = minf(r, minf(g, b))
		sat += (mx - mn) / mx if mx > 0.0 else 0.0
		n += 1
	if n == 0:
		return {"mean": 0.0, "std": 0.0, "sat": 0.0, "n": 0}
	var mean: float = s / n
	return {"mean": mean, "std": sqrt(maxf(0.0, s2 / n - mean * mean)), "sat": sat / n, "n": n}


static func _apply_quiet(data: PackedByteArray, contrast: float, gray_mean: float, target_mean: float, factor: float, stride: int) -> PackedByteArray:
	var out: PackedByteArray = data.duplicate()
	var step: int = 4 * maxi(1, stride)
	for i: int in range(0, data.size(), step):
		var r: float = data[i] / 255.0
		var g: float = data[i + 1] / 255.0
		var b: float = data[i + 2] / 255.0
		var gray: float = LR * r + LG * g + LB * b
		var q: float = clampf((gray - gray_mean) * contrast + target_mean, 0.0, 1.0)
		out[i] = int(roundf(clampf(q + (r - gray) * contrast * factor, 0.0, 1.0) * 255.0))
		out[i + 1] = int(roundf(clampf(q + (g - gray) * contrast * factor, 0.0, 1.0) * 255.0))
		out[i + 2] = int(roundf(clampf(q + (b - gray) * contrast * factor, 0.0, 1.0) * 255.0))
	return out


## The ground cap (tone.py `cap_ground`): returns {image, before, after, contrast, chroma, passed}.
static func cap_ground(src: Image, person_std: float, mean_max: float, std_ratio: float, sat_max: float, margin: float = 0.9) -> Dictionary:
	var img: Image = src.duplicate()
	img.convert(Image.FORMAT_RGBA8)
	var before: Dictionary = metrics(img)
	var target_mean: float = minf(before["mean"], mean_max * margin)
	var target_std: float = person_std * std_ratio * margin
	var contrast: float = minf(1.0, target_std / maxf(before["std"], 1e-12))
	var data: PackedByteArray = img.get_data()
	var w: int = img.get_width()
	var h: int = img.get_height()
	# The chroma factor's binary search reads every 3rd pixel (the result is re-measured on all pixels below).
	var low: float = 0.0
	var high: float = 1.0
	for _i: int in 18:
		var mid: float = (low + high) * 0.5
		var trial: Image = Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, _apply_quiet(data, contrast, before["mean"], target_mean, mid, 3))
		if metrics(trial, 3)["sat"] <= sat_max * margin:
			low = mid
		else:
			high = mid
	var final: Image = Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, _apply_quiet(data, contrast, before["mean"], target_mean, low, 1))
	var after: Dictionary = metrics(final)
	var passed: bool = after["mean"] <= mean_max and after["std"] <= person_std * std_ratio and after["sat"] <= sat_max
	return {"image": final, "before": before, "after": after, "contrast": contrast, "chroma": low, "passed": passed}


## Winter (`mix` = kept saturation, 0.35) or drought (`tint` toward ochre at `mix`); other kinds return a copy.
static func season(src: Image, kind: String, mix: float, tint: Array = []) -> Image:
	var img: Image = src.duplicate()
	img.convert(Image.FORMAT_RGBA8)
	if kind != "winter" and kind != "drought":
		return img
	var data: PackedByteArray = img.get_data()
	var tr: float = float(tint[0]) if tint.size() == 3 else 1.25
	var tg: float = float(tint[1]) if tint.size() == 3 else 1.04
	var tb: float = float(tint[2]) if tint.size() == 3 else 0.65
	for i: int in range(0, data.size(), 4):
		var r: float = data[i] / 255.0
		var g: float = data[i + 1] / 255.0
		var b: float = data[i + 2] / 255.0
		var gray: float = LR * r + LG * g + LB * b
		var o: Vector3
		if kind == "winter":
			o = Vector3(gray + (r - gray) * mix, gray + (g - gray) * mix, gray + (b - gray) * mix)
		else:
			o = Vector3(r * (1.0 - mix) + gray * tr * mix, g * (1.0 - mix) + gray * tg * mix, b * (1.0 - mix) + gray * tb * mix)
		data[i] = int(roundf(clampf(o.x, 0.0, 1.0) * 255.0))
		data[i + 1] = int(roundf(clampf(o.y, 0.0, 1.0) * 255.0))
		data[i + 2] = int(roundf(clampf(o.z, 0.0, 1.0) * 255.0))
	return Image.create_from_data(img.get_width(), img.get_height(), false, Image.FORMAT_RGBA8, data)


## Composite `snow` (tiled from the image's origin) over `src` with its alpha clamped to `alpha_max` (0..1).
static func add_snow(src: Image, snow: Image, alpha_max: float) -> Image:
	var img: Image = src.duplicate()
	img.convert(Image.FORMAT_RGBA8)
	if snow == null:
		return img
	var s: Image = snow.duplicate()
	s.convert(Image.FORMAT_RGBA8)
	var sd: PackedByteArray = s.get_data()
	var cap: int = int(roundf(clampf(alpha_max, 0.0, 1.0) * 255.0))
	for i: int in range(3, sd.size(), 4):
		sd[i] = mini(sd[i], cap)
	s = Image.create_from_data(s.get_width(), s.get_height(), false, Image.FORMAT_RGBA8, sd)
	for y: int in range(0, img.get_height(), s.get_height()):
		for x: int in range(0, img.get_width(), s.get_width()):
			img.blend_rect(s, Rect2i(0, 0, s.get_width(), s.get_height()), Vector2i(x, y))
	return img


static func _lin(c: float) -> float:
	return c / 12.92 if c <= 0.04045 else pow((c + 0.055) / 1.055, 2.4)


static func _enc(c: float) -> float:
	c = clampf(c, 0.0, 1.0)
	return c * 12.92 if c <= 0.0031308 else 1.055 * pow(c, 1.0 / 2.4) - 0.055


static func _f(t: float) -> float:
	var d: float = 6.0 / 29.0
	return pow(t, 1.0 / 3.0) if t > d * d * d else t / (3.0 * d * d) + 4.0 / 29.0


static func _finv(t: float) -> float:
	var d: float = 6.0 / 29.0
	return t * t * t if t > d else 3.0 * d * d * (t - 4.0 / 29.0)


## sRGB (0..1) → CIELAB D65.
static func rgb_to_lab(c: Vector3) -> Vector3:
	var r: float = _lin(c.x)
	var g: float = _lin(c.y)
	var b: float = _lin(c.z)
	var x: float = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047
	var y: float = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b
	var z: float = (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883
	var fx: float = _f(x)
	var fy: float = _f(y)
	var fz: float = _f(z)
	return Vector3(116.0 * fy - LAB_L_OFFSET, 500.0 * (fx - fy), 200.0 * (fy - fz))


## CIELAB D65 → sRGB (0..1), clipped.
static func lab_to_rgb(lab: Vector3) -> Vector3:
	var fy: float = (lab.x + LAB_L_OFFSET) / 116.0
	var x: float = _finv(fy + lab.y / 500.0) * 0.95047
	var y: float = _finv(fy)
	var z: float = _finv(fy - lab.z / 200.0) * 1.08883
	var r: float = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z
	var g: float = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z
	var b: float = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z
	return Vector3(_enc(r), _enc(g), _enc(b))


## A declared Lab shift on every pixel; alpha unchanged (art/materials/tone.py `shift_image`).
static func lab_shift(src: Image, d_l: float, d_a: float, d_b: float) -> Image:
	var img: Image = src.duplicate()
	img.convert(Image.FORMAT_RGBA8)
	if d_l == 0.0 and d_a == 0.0 and d_b == 0.0:
		return img
	var data: PackedByteArray = img.get_data()
	var delta: Vector3 = Vector3(d_l, d_a, d_b)
	for i: int in range(0, data.size(), 4):
		var lab: Vector3 = rgb_to_lab(Vector3(data[i] / 255.0, data[i + 1] / 255.0, data[i + 2] / 255.0)) + delta
		var rgb: Vector3 = lab_to_rgb(lab)
		data[i] = int(roundf(rgb.x * 255.0))
		data[i + 1] = int(roundf(rgb.y * 255.0))
		data[i + 2] = int(roundf(rgb.z * 255.0))
	return Image.create_from_data(img.get_width(), img.get_height(), false, Image.FORMAT_RGBA8, data)


## The wall top's tone: a periodic (wrapping) Gaussian blur of `blur_px` sigma, then saturation (blend with the
## ITU-R 601 grey, as PIL's ImageEnhance.Color) and value; alpha unchanged.
static func top_tone(src: Image, value: float, saturation: float, blur_px: float) -> Image:
	var img: Image = src.duplicate()
	img.convert(Image.FORMAT_RGBA8)
	var w: int = img.get_width()
	var h: int = img.get_height()
	var data: PackedByteArray = img.get_data()
	var alpha: PackedByteArray = PackedByteArray()
	alpha.resize(w * h)
	for p: int in w * h:
		alpha[p] = data[p * 4 + 3]
	if blur_px > 0.0:
		var radius: int = ceili(blur_px * 3.0)
		var kernel: PackedFloat32Array = PackedFloat32Array()
		var total: float = 0.0
		for k: int in range(-radius, radius + 1):
			var v: float = exp(-float(k * k) / (2.0 * blur_px * blur_px))
			kernel.append(v)
			total += v
		for k: int in kernel.size():
			kernel[k] /= total
		data = _blur_pass(data, w, h, kernel, radius, true)
		data = _blur_pass(data, w, h, kernel, radius, false)
	for i: int in range(0, data.size(), 4):
		var r: float = data[i]
		var g: float = data[i + 1]
		var b: float = data[i + 2]
		var grey: float = (r * 299.0 + g * 587.0 + b * 114.0) / 1000.0
		r = clampf(grey + (r - grey) * saturation, 0.0, 255.0)
		g = clampf(grey + (g - grey) * saturation, 0.0, 255.0)
		b = clampf(grey + (b - grey) * saturation, 0.0, 255.0)
		data[i] = int(roundf(r * value))
		data[i + 1] = int(roundf(g * value))
		data[i + 2] = int(roundf(b * value))
		data[i + 3] = alpha[i >> 2]
	return Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, data)


static func _blur_pass(data: PackedByteArray, w: int, h: int, kernel: PackedFloat32Array, radius: int, horizontal: bool) -> PackedByteArray:
	var out: PackedByteArray = data.duplicate()
	for y: int in h:
		for x: int in w:
			var acc: Vector4 = Vector4.ZERO
			for k: int in range(-radius, radius + 1):
				var sx: int = posmod(x + k, w) if horizontal else x
				var sy: int = y if horizontal else posmod(y + k, h)
				var j: int = (sy * w + sx) * 4
				var wgt: float = kernel[k + radius]
				acc += Vector4(data[j], data[j + 1], data[j + 2], data[j + 3]) * wgt
			var i: int = (y * w + x) * 4
			out[i] = int(roundf(acc.x))
			out[i + 1] = int(roundf(acc.y))
			out[i + 2] = int(roundf(acc.z))
			out[i + 3] = int(roundf(acc.w))
	return out

# tone.gd — pixel and colour transforms of the M-ART2 art, applied once per texture at load (never per frame).
# Purpose: ports of the art line's tone code (v8/art-polish2 c8ed8608): `lab_shift` (art/materials/tone.py
#   `shift_image`: a D65 CIELAB shift, alpha kept; the wall and floor materials of the build menu's icons),
#   `rgb_to_lab` / `lab_to_rgb` (item icon tints) and `top_tone` (the wall top's periodic blur, saturation and value).
#   Luminance is Rec.709 on encoded sRGB, as on the art line (LR/LG/LB; the minimap's dots). Every image function takes
#   and returns RGBA8 Images and is deterministic (no randomness, fixed iteration counts). M3D-SWITCH E2 removed the
#   2D ground's cap, season and snow transforms with the ground chunks.
# Bridge methods called: none.
extends RefCounted

const LR: float = 0.2126
const LG: float = 0.7152
const LB: float = 0.0722
## CIELAB: L* = 116 f(Y) − 16 (a colour-science constant, not a tile size).
const LAB_L_OFFSET: float = 116.0 - 100.0


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

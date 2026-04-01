shader_type canvas_item;

// Existing textures
uniform sampler2D band_id_tex : filter_nearest, repeat_disable;
uniform sampler2D density_tex : filter_linear, repeat_disable;

// Per-tile hardness (0=soft heatmap, 255=crisp political boundary)
uniform sampler2D hardness_tex : filter_linear, repeat_disable;
// Per-tile dispute intensity (0=no dispute, 255=strong dispute)
uniform sampler2D dispute_tex : filter_linear, repeat_disable;

uniform vec3 band_colors[8];
uniform int band_count = 0;

uniform float fill_alpha_min : hint_range(0.0, 1.0) = 0.10;
uniform float fill_alpha_max : hint_range(0.0, 1.0) = 0.45;
uniform float border_alpha_min : hint_range(0.0, 1.0) = 0.30;
uniform float border_alpha_max : hint_range(0.0, 1.0) = 0.85;
uniform float density_threshold : hint_range(0.0, 1.0) = 0.08;

// 0.0 = zoomed out (Z4-Z5), 1.0 = zoomed in (Z1-Z2)
uniform float zoom_factor : hint_range(0.0, 1.0) = 0.5;

uniform float stripe_width = 6.0;
uniform vec3 dispute_color = vec3(0.9, 0.2, 0.2);
uniform float dispute_alpha : hint_range(0.0, 1.0) = 0.35;

void fragment() {
	float density = texture(density_tex, UV).r;
	float band_id_raw = texture(band_id_tex, UV).r;
	int band_id = int(band_id_raw * 255.0 + 0.5) - 1;

	if (density < density_threshold || band_id < 0 || band_id >= band_count) {
		float dispute = texture(dispute_tex, UV).r;
		if (dispute > 0.1) {
			float stripe = mod(FRAGCOORD.x + FRAGCOORD.y, stripe_width * 2.0);
			float stripe_mask = step(stripe_width, stripe);
			COLOR = vec4(dispute_color, dispute * dispute_alpha * stripe_mask);
		} else {
			COLOR = vec4(0.0);
		}
		return;
	}

	vec3 color = band_colors[band_id];
	float hardness = texture(hardness_tex, UV).r;

	// Border detection: compare faction ID with 4 axis-aligned neighbors
	vec2 texel = vec2(1.0) / vec2(textureSize(band_id_tex, 0));
	int n_up    = int(texture(band_id_tex, UV + vec2(0.0,     -texel.y)).r * 255.0 + 0.5) - 1;
	int n_down  = int(texture(band_id_tex, UV + vec2(0.0,      texel.y)).r * 255.0 + 0.5) - 1;
	int n_left  = int(texture(band_id_tex, UV + vec2(-texel.x, 0.0    )).r * 255.0 + 0.5) - 1;
	int n_right = int(texture(band_id_tex, UV + vec2( texel.x, 0.0    )).r * 255.0 + 0.5) - 1;

	bool is_border = (n_up != band_id) || (n_down != band_id) ||
	                 (n_left != band_id) || (n_right != band_id);

	// At high hardness, expand border detection to diagonal neighbors (thicker borders)
	bool is_thick_border = false;
	if (hardness > 0.5) {
		int n_ul = int(texture(band_id_tex, UV + vec2(-texel.x, -texel.y)).r * 255.0 + 0.5) - 1;
		int n_ur = int(texture(band_id_tex, UV + vec2( texel.x, -texel.y)).r * 255.0 + 0.5) - 1;
		int n_dl = int(texture(band_id_tex, UV + vec2(-texel.x,  texel.y)).r * 255.0 + 0.5) - 1;
		int n_dr = int(texture(band_id_tex, UV + vec2( texel.x,  texel.y)).r * 255.0 + 0.5) - 1;
		is_thick_border = (n_ul != band_id) || (n_ur != band_id) ||
		                  (n_dl != band_id) || (n_dr != band_id);
	}

	// Density-to-alpha: narrow smoothstep at high hardness → sharper edge
	float edge_range = mix(0.5, 0.15, hardness);
	float density_alpha = smoothstep(density_threshold, density_threshold + edge_range, density);

	// Hardness-modulated alpha
	float fill_alpha   = mix(fill_alpha_min,   fill_alpha_max,   hardness);
	float border_alpha = mix(border_alpha_min, border_alpha_max, hardness);

	// Zoom adaptation: slightly boost opacity when zoomed out for map readability
	fill_alpha   *= mix(1.3, 1.0, zoom_factor);
	border_alpha *= mix(1.5, 1.0, zoom_factor);

	float dispute = texture(dispute_tex, UV).r;
	bool is_disputed = dispute > 0.1;

	if (is_border || (is_thick_border && hardness > 0.5)) {
		vec3 border_col = color * 0.7;
		if (is_disputed) {
			float stripe = mod(FRAGCOORD.x - FRAGCOORD.y, stripe_width * 2.0);
			float stripe_mask = step(stripe_width, stripe);
			border_col = mix(border_col, dispute_color, stripe_mask * dispute);
		}
		COLOR = vec4(border_col, clamp(border_alpha * density_alpha, 0.0, 1.0));
	} else {
		vec3 fill_col = color;
		if (is_disputed) {
			float stripe = mod(FRAGCOORD.x + FRAGCOORD.y, stripe_width * 2.0);
			float stripe_mask = smoothstep(stripe_width - 1.0, stripe_width + 1.0, stripe);
			fill_col = mix(fill_col, dispute_color, stripe_mask * dispute * 0.4);
		}
		COLOR = vec4(fill_col, clamp(fill_alpha * density_alpha, 0.0, 1.0));
	}
}

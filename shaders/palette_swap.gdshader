shader_type canvas_item;

// V7 Phase 4-γ — per-instance palette indices via INSTANCE_CUSTOM.
//
// The previous three `uniform float *_index` declarations have been
// replaced by per-instance custom data so MultiMeshInstance2D can drive
// hundreds/thousands of agents with distinct palettes in a single
// draw call.
//
// Channel layout of INSTANCE_CUSTOM (normalised 0.0..=1.0):
//   .r — hair column index ∈ [0, 8) (palette row 0, 8 columns)
//   .g — body column index ∈ [0, 4) (palette row 1, 4 columns)
//   .b — skin column index ∈ [0, 8) (palette row 2, 8 columns)
//
// The renderer is expected to write `Vector4(hair/7, body/3, skin/7, 0)`
// per instance via `MultiMesh.set_instance_custom_data` so the rounded
// integer column indices match the 8/4/8 palette layout.

uniform sampler2D palette_lut : filter_nearest, repeat_disable;

varying flat vec3 v_palette;

vec2 palette_uv(float row_index, float column_index) {
	return vec2((column_index + 0.5) / 8.0, (row_index + 0.5) / 3.0);
}

void vertex() {
	// Forward the per-instance custom-data .rgb to the fragment stage.
	v_palette = INSTANCE_CUSTOM.rgb;
}

void fragment() {
	// V7 Phase 11-α — capture instance color (state_tag tint) before
	// overwriting COLOR with the palette-swapped result. When
	// MultiMesh.use_colors is true, COLOR on entry carries the per-instance
	// color written by set_instance_color(); we save it here and multiply
	// at the end so the tint composites correctly over the palette output.
	vec4 modulate = COLOR;
	vec4 tex = texture(TEXTURE, UV);
	if (tex.a <= 0.01) {
		COLOR = tex;
	} else {
		float hair_col = floor(v_palette.r * 7.0 + 0.5);
		float body_col = floor(v_palette.g * 3.0 + 0.5);
		float skin_col = floor(v_palette.b * 7.0 + 0.5);

		float row_selector = tex.g;
		float row_index = 1.0;
		float column_index = body_col;

		if (row_selector < 0.25) {
			row_index = 0.0;
			column_index = hair_col;
		} else if (row_selector < 0.75) {
			row_index = 1.0;
			column_index = body_col;
		} else {
			row_index = 2.0;
			column_index = skin_col;
		}

		vec4 palette_color = texture(palette_lut, palette_uv(row_index, column_index));
		COLOR = vec4(palette_color.rgb * modulate.rgb, tex.a);
	}
}

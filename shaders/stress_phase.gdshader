shader_type canvas_item;

uniform sampler2D palette_lut : filter_nearest, repeat_disable;
uniform float hair_index : hint_range(0.0, 7.0, 1.0) = 0.0;
uniform float body_index : hint_range(0.0, 3.0, 1.0) = 0.0;
uniform float skin_index : hint_range(0.0, 7.0, 1.0) = 0.0;

uniform int stress_phase : hint_range(0, 4) = 0;
uniform vec4 mood_color : source_color = vec4(0.3, 0.8, 0.3, 1.0);
uniform vec4 outline_color : source_color = vec4(1.0, 0.6, 0.0, 1.0);
uniform bool active_break = false;
uniform float outline_width : hint_range(0.0, 10.0) = 2.0;

uniform float breath_amplitude : hint_range(0.0, 3.0) = 0.8;
uniform float breath_speed : hint_range(0.5, 5.0) = 1.5;
uniform float agent_offset = 0.0;
uniform float speed_factor : hint_range(0.0, 2.0) = 1.0;

float hash11(float p) {
	return fract(sin(p * 12.9898) * 43758.5453);
}

float dash_frequency() {
	return stress_phase >= 2 ? 15.0 : 10.0;
}

float dash_speed() {
	return stress_phase >= 2 ? 7.5 : 3.0;
}

float shake_intensity() {
	if (stress_phase >= 4) {
		return 3.0;
	}
	if (stress_phase >= 2) {
		return 1.5;
	}
	return 0.0;
}

float shake_interval() {
	return stress_phase >= 4 ? 0.03 : 0.06;
}

vec2 palette_uv(float row_index, float column_index) {
	return vec2((column_index + 0.5) / 8.0, (row_index + 0.5) / 3.0);
}

bool inside_sprite(vec2 uv) {
	return all(greaterThanEqual(uv, vec2(0.0))) && all(lessThanEqual(uv, vec2(1.0)));
}

void vertex() {
	if (stress_phase >= 1) {
		VERTEX += (UV * 2.0 - 1.0) * outline_width;
	}

	float sway_strength = smoothstep(0.3, 0.0, speed_factor);
	float breath = sin((TIME * breath_speed + agent_offset) * 6.28318530718) * breath_amplitude * sway_strength;
	float lean = sin((TIME * breath_speed * 0.7 + agent_offset + 2.0) * 6.28318530718) * breath_amplitude * 0.3 * sway_strength;
	VERTEX.y += breath;
	VERTEX.x += lean;

	if (stress_phase >= 2) {
		float intensity = shake_intensity();
		float interval = shake_interval();
		float chunk = floor(TIME / interval);
		VERTEX.x += (hash11(VERTEX.x + chunk) * 2.0 - 1.0) * intensity;
		VERTEX.y += (hash11(VERTEX.y + chunk + 7.0) * 2.0 - 1.0) * intensity;
	}

	if (stress_phase >= 4) {
		VERTEX *= 1.0 + 0.06 * sin(TIME * 12.56637061436);
	}
}

void fragment() {
	vec2 uv = UV;
	if (stress_phase >= 1) {
		vec2 expanded_size = vec2(1.0) + TEXTURE_PIXEL_SIZE * outline_width * 2.0;
		uv = (uv * expanded_size) - TEXTURE_PIXEL_SIZE * outline_width;
	}

	vec4 base_color = vec4(0.0);
	if (inside_sprite(uv)) {
		vec2 sample_uv = clamp(uv, vec2(0.0), vec2(1.0));
		vec4 tex = texture(TEXTURE, sample_uv);
		if (tex.a > 0.01) {
			float row_selector = tex.g;
			float row_index = 1.0;
			float column_index = body_index;
			if (row_selector < 0.25) {
				row_index = 0.0;
				column_index = hair_index;
			} else if (row_selector >= 0.75) {
				row_index = 2.0;
				column_index = skin_index;
			}
			vec4 palette_color = texture(palette_lut, palette_uv(row_index, column_index));
			base_color = vec4(palette_color.rgb, tex.a);
		}
	}
	COLOR = base_color;

	bool outlined = false;
	if (stress_phase >= 1 && base_color.a <= 0.01) {
		bool near_edge = false;
		for (float i = -4.0; i <= 4.0 && !near_edge; i += 1.0) {
			for (float j = -4.0; j <= 4.0 && !near_edge; j += 1.0) {
				if (abs(i) > outline_width || abs(j) > outline_width) {
					continue;
				}
				vec2 sample_uv = clamp(uv + TEXTURE_PIXEL_SIZE * vec2(i, j), vec2(0.0), vec2(1.0));
				if (texture(TEXTURE, sample_uv).a > 0.1) {
					near_edge = true;
				}
			}
		}

		if (near_edge) {
			vec2 pos = uv - 0.5;
			float angle = atan(pos.y, pos.x);
			float dash = step(0.5, 0.5 * sin(dash_frequency() * angle + TIME * dash_speed()) + 0.5);
			COLOR = vec4(outline_color.rgb, outline_color.a * dash);
			outlined = true;
		}
	}

	if (!outlined && COLOR.a > 0.01) {
		COLOR.rgb = mix(COLOR.rgb, mood_color.rgb, 0.18);

		if (stress_phase == 3) {
			COLOR.a *= mix(0.3, 1.0, 0.5 + 0.5 * sin(TIME * 25.13274122872));
			float gray = dot(COLOR.rgb, vec3(0.299, 0.587, 0.114));
			COLOR.rgb = mix(COLOR.rgb, vec3(gray), 0.3);
		}

		if (stress_phase >= 4) {
			COLOR.rgb = mix(COLOR.rgb, vec3(1.0, 0.1, 0.0), 0.5 + 0.3 * sin(TIME * 3.0));
			COLOR.a *= 0.7 + 0.3 * sin(TIME * 10.0);
		}

		if (active_break) {
			float flash = step(0.5, fract(TIME * 8.0));
			COLOR.rgb = mix(vec3(1.0, 0.0, 0.0), vec3(1.0), flash);
			COLOR.a = max(COLOR.a, 0.6);
		}
	}
}

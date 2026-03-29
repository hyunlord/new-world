shader_type canvas_item;

uniform sampler2D band_id_tex : filter_nearest, repeat_disable;
uniform sampler2D density_tex : filter_linear, repeat_disable;
uniform vec3 band_colors[8];
uniform int band_count = 0;
uniform float fill_alpha : hint_range(0.0, 1.0) = 0.20;
uniform float border_alpha : hint_range(0.0, 1.0) = 0.60;
uniform float density_threshold : hint_range(0.0, 1.0) = 0.08;

void fragment() {
    float density = texture(density_tex, UV).r;
    float band_id_raw = texture(band_id_tex, UV).r;
    int band_id = int(band_id_raw * 255.0 + 0.5) - 1;

    if (density < density_threshold || band_id < 0 || band_id >= band_count) {
        COLOR = vec4(0.0);
    } else {
        vec3 color = band_colors[band_id];

        vec2 texel = vec2(1.0) / vec2(textureSize(band_id_tex, 0));
        int n_up = int(texture(band_id_tex, UV + vec2(0, -texel.y)).r * 255.0 + 0.5) - 1;
        int n_down = int(texture(band_id_tex, UV + vec2(0, texel.y)).r * 255.0 + 0.5) - 1;
        int n_left = int(texture(band_id_tex, UV + vec2(-texel.x, 0)).r * 255.0 + 0.5) - 1;
        int n_right = int(texture(band_id_tex, UV + vec2(texel.x, 0)).r * 255.0 + 0.5) - 1;

        bool is_border = (n_up != band_id) || (n_down != band_id) ||
                         (n_left != band_id) || (n_right != band_id);

        float density_alpha = smoothstep(density_threshold, 0.5, density);

        if (is_border) {
            COLOR = vec4(color, border_alpha * density_alpha);
        } else {
            COLOR = vec4(color, fill_alpha * density_alpha);
        }
    }
}

shader_type canvas_item;

uniform sampler2D data_texture : filter_linear, repeat_disable;
uniform float overlay_opacity : hint_range(0.0, 1.0) = 0.55;
uniform vec4 color_low = vec4(0.1, 0.2, 0.8, 1.0);
uniform vec4 color_mid_low = vec4(0.1, 0.7, 0.7, 1.0);
uniform vec4 color_mid = vec4(0.2, 0.8, 0.2, 1.0);
uniform vec4 color_mid_high = vec4(0.9, 0.9, 0.1, 1.0);
uniform vec4 color_high = vec4(0.9, 0.15, 0.1, 1.0);
uniform float threshold : hint_range(0.0, 1.0) = 0.03;

void fragment() {
    float value = texture(data_texture, UV).r;
    if (value < threshold) {
        COLOR = vec4(0.0);
    } else {
        vec4 gradient_color;
        if (value < 0.25) {
            gradient_color = mix(color_low, color_mid_low, value / 0.25);
        } else if (value < 0.5) {
            gradient_color = mix(color_mid_low, color_mid, (value - 0.25) / 0.25);
        } else if (value < 0.75) {
            gradient_color = mix(color_mid, color_mid_high, (value - 0.5) / 0.25);
        } else {
            gradient_color = mix(color_mid_high, color_high, (value - 0.75) / 0.25);
        }

        // Value-dependent alpha: low values more transparent, high values stronger
        float value_alpha = smoothstep(threshold, 0.5, value) * 0.6 + 0.4;
        COLOR = vec4(gradient_color.rgb, overlay_opacity * value_alpha * gradient_color.a);
    }
}

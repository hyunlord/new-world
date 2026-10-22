shader_type canvas_item;
uniform vec4 replacement : source_color = vec4(0.65, 0.43, 0.27, 1.0);
uniform float neutral_value = 0.69;
uniform float tolerance = 0.13;
varying vec4 instance_tint;
void vertex() { instance_tint = COLOR; }
void fragment() {
    vec4 tex = texture(TEXTURE, UV);
    float spread = max(tex.r, max(tex.g, tex.b)) - min(tex.r, min(tex.g, tex.b));
    float neutral = (1.0 - smoothstep(0.025, 0.09, spread)) *
        (1.0 - smoothstep(tolerance * 0.5, tolerance, abs(tex.r - neutral_value)));
    vec3 tinted = replacement.rgb * tex.rgb / max(neutral_value, 0.01);
    COLOR = vec4(mix(tex.rgb, tinted, neutral), tex.a) * instance_tint;
}

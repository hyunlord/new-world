// Preserve generated ink and wash luminance while applying requested skin/clothes colours.
// Bridge methods: none. This is continuous tinting, not palette quantization.
shader_type canvas_item;
uniform vec4 replacement : source_color = vec4(0.65, 0.43, 0.27, 1.0);
uniform float reference_luma = 0.65;
uniform bool skin_only = false;
varying vec4 instance_tint;
void vertex() { instance_tint = COLOR; }
void fragment() {
    vec4 tex = texture(TEXTURE, UV);
    float luma = dot(tex.rgb, vec3(0.2126, 0.7152, 0.0722));
    float ink_preservation = smoothstep(0.12, 0.30, luma);
    float warm_skin = smoothstep(0.07, 0.18, tex.r - tex.b) * smoothstep(0.35, 0.50, luma);
    float mask = ink_preservation * (skin_only ? warm_skin : 1.0);
    vec3 wash = replacement.rgb * (luma / max(reference_luma, 0.01));
    COLOR = vec4(mix(tex.rgb, wash, mask), tex.a) * instance_tint;
}

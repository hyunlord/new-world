shader_type canvas_item;
uniform int season : hint_range(0, 3) = 1;
varying vec4 instance_tint;
vec3 hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0/3.0, 2.0/3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    float d = q.x - min(q.w, q.y);
    return vec3(abs(q.z + (q.w-q.y)/(6.0*d+0.000001)), d/(q.x+0.000001), q.x);
}
vec3 rgb(vec3 c) {
    vec3 p = abs(fract(c.xxx + vec3(0.0, 2.0/3.0, 1.0/3.0))*6.0-3.0);
    return c.z * mix(vec3(1.0), clamp(p-1.0, 0.0, 1.0), c.y);
}
void vertex() { instance_tint = COLOR; }
void fragment() {
    vec4 tex = texture(TEXTURE, UV);
    vec3 c = hsv(tex.rgb);
    // Four fixed HSV curve presets: spring, summer, autumn, winter.
    if (season == 0) { c.y *= 0.90; c.z = pow(c.z, 0.92); }
    if (season == 2) { c.x = mix(c.x, 0.075, smoothstep(0.08, 0.18, c.x) * (1.0-smoothstep(0.36, 0.5, c.x)) * 0.75); c.y *= 0.83; }
    if (season == 3) { c.y *= 0.18; c.z = mix(c.z, pow(c.z, 0.52), 0.8); }
    COLOR = vec4(rgb(c), tex.a) * instance_tint;
}

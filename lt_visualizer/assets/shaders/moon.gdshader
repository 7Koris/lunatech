shader_type spatial;

uniform sampler2D _a;
uniform sampler2D heightmap;
uniform float height_ratio = 2.0;
uniform float start = 0;
uniform float end = 0.1;

void vertex() {
	vec2 uv_range = vec2(end, end) - vec2(start, start);
    vec2 adjusted_uv = vec2(start, start) + uv_range * UV; // Remap UVs to the subsection
	VERTEX.y += texture(heightmap, adjusted_uv).r * height_ratio;
}

void fragment() {
	vec2 uv_range = vec2(end, end) - vec2(start, start);
    vec2 adjusted_uv = vec2(start, start) + uv_range * UV; // Remap UVs to the subsection
	ALBEDO = texture(_a, adjusted_uv).rgb;
}

#version 400 core

in vec2 uv;
in float light;
in float fogFactor;

uniform vec3 fogColor;
uniform sampler2D textureSampler;

out vec4 Color;

void main()
{
    vec4 col = texture(textureSampler, uv);
    if (col.a == 0.0) discard;
    col.xyz *= light;
    Color = mix(vec4(fogColor,1.0), col, fogFactor);
}
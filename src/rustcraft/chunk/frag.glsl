#version 400 core

in vec2 uv;
in float light;

uniform sampler2D textureSampler;

out vec4 Color;

void main()
{
    vec4 col = texture(textureSampler, uv);
    if (col.a < 1.0) discard;
    col.xyz *= light;
    Color = col;
}
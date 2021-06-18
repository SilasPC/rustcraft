#version 400 core

in vec2 uv;

uniform sampler2D textureSampler;

out vec4 Color;

void main()
{
    vec4 col = texture(textureSampler, uv);
    Color = col;
}
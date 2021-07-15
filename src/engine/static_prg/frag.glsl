#version 400 core

in vec2 uv;

uniform float lightScale;
uniform sampler2D textureSampler;

out vec4 Color;

void main()
{
    vec4 col = texture(textureSampler, uv);
    if (col.a == 0.0) discard;
    Color = vec4(lightScale * col.xyz, col.a);
}
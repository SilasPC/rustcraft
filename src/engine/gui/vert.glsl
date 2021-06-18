#version 400 core

uniform vec2 scale;
uniform vec2 position;

layout (location = 0) in vec3 vert;
layout (location = 1) in vec2 uvIn;

out vec2 uv;

void main()
{
    uv = uvIn;
    gl_Position = vec4(position, 0., 0.) + vec4(vert.x * scale.x, vert.y * scale.y, vert.z, 1.0);
}
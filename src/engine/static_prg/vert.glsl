#version 400 core

uniform mat4 transform;
uniform mat4 view;
uniform mat4 project;
uniform vec2 uvScale;
uniform vec2 uvOffset;

layout (location = 0) in vec3 vert;
layout (location = 1) in vec2 uvIn;

out vec2 uv;

void main()
{
    uv = uvScale.xy * uvIn.xy + uvOffset;
    gl_Position = project * view * transform * vec4(vert, 1.0);
}
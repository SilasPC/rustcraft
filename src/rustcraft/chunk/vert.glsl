#version 400 core

uniform mat4 transform;
uniform mat4 view;
uniform mat4 project;
uniform float globLight;

layout (location = 0) in vec3 vert;
layout (location = 1) in vec2 uvIn;
layout (location = 2) in float lightIn;

out vec2 uv;
out float light;

void main()
{
    uv = uvIn;
    light = max(globLight,lightIn);
    gl_Position = project * view * transform * vec4(vert, 1.0);
}
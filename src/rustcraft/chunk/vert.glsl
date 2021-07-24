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
out float fogFactor;

// very close = {den=0.3, gra=2.5}
const float fogDensity = 0.03;
const float fogGradient = 2.5;

void main()
{
    uv = uvIn;
    light = max(globLight,lightIn);

    vec4 worldPos = transform * vec4(vert, 1.0);
    vec4 viewPos = view * worldPos;
    gl_Position = project * viewPos;
    
    float distanceToCamera = length(viewPos.xyz);
    fogFactor = exp(-pow(distanceToCamera*fogDensity, fogGradient));
    fogFactor = clamp(fogFactor, 0.0, 1.0);
}
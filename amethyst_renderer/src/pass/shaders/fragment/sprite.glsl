// TODO: Needs documentation.

#version 150 core

uniform sampler2D albedo;

layout (std140) uniform  Locals {
    vec4 mul_color;
};

in vec2 tex_uv;

out vec4 color;

void main() {
    color = texture(albedo, tex_uv) * mul_color;
}

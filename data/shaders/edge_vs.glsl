#version 460

uniform mat4 ViewProj;
uniform vec3 Positions[2];

void main() {
    gl_Position = ViewProj * vec4(Positions[gl_VertexID], 1.0);
}


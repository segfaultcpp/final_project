#version 460

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

uniform mat4 Model;
uniform mat4 ViewProj;

out vec3 frag_position;
out vec3 frag_normal;

void main() {
    mat3 normal_mat = transpose(inverse(mat3(Model)));
    frag_normal = normal_mat * aNormal;
    
    frag_position = (Model * vec4(aPos, 1.0)).xyz;
    gl_Position = ViewProj * vec4(frag_position, 1.0);
}


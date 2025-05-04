#version 460 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoords;

out vec2 tex_coords;

void main()
{
    gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0); 
    tex_coords = vec2(aTexCoords.x, 1.0 - aTexCoords.y);
}  

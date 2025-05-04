#version 460 core
out vec4 out_color;
  
in vec2 tex_coords;

uniform sampler2D FullscreenTex;

void main()
{ 
    out_color = texture(FullscreenTex, tex_coords);
}

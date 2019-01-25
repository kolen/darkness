#version 150 core

uniform sampler2D t_Dif;

in vec2 v_Uv;
out vec4 o_Color;

void main()
{
  vec3 dif = vec3(texture(t_Dif, v_Uv).r);

  vec3 result = vec3(0.0);
  // manipulations here
  result = dif;

  o_Color = vec4(dif, 1.0);
}

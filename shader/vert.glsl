#version 150 core

in vec4 a_Pos;
in vec2 a_Uv;
out vec2 v_Uv;

uniform mat4 u_Transform;

void main()
{
    v_Uv = a_Uv;
    gl_Position = u_Transform * a_Pos;
}

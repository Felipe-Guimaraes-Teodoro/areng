#version 460
layout(location = 0) out vec4 f_color;
layout(location = 1) in vec3 s_color;

void main() {
    f_color = vec4(s_color, 1.0);
}

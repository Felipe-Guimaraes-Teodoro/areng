#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

// per instance data
//layout(location = 2) in vec3 ofs;

// layout(location = 1) out vec3 s_color;

void main() {
  // vec2 outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
  // gl_Position = vec4(outUV * 2.0 - 1.0, 0.0, 1.0);

  // pos = vec3(outUV * 2.0 - 1.0, 0.0);
  //

  gl_Position = vec4(position, 1.0);
  s_color = color;
}


// vertex shader
pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        // path: "src/shaders/vert.vert"
        src: r#"
#version 460

layout(location = 0) in vec3 position;

layout(push_constant) uniform PushConstantCameraData {
    mat4 proj;
    mat4 view;
};

void main() {
  // vec2 outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
  // gl_Position = vec4(outUV * 2.0 - 1.0, 0.0, 1.0);

  // pos = vec3(outUV * 2.0 - 1.0, 0.0);
  //

  gl_Position = proj * view * vec4(position, 1.0);
}
        "#,
    }
}

// fragment shader
pub mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "src/shaders/frag.frag"
    }
}

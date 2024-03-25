
// vertex shader
pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        // path: "src/shaders/vert.vert"
        src: r#"
#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 ofs;
layout(location = 2) in vec3 fun_factor;

// per instance data
layout(location = 3) in vec3 color;

layout(location = 1) out vec3 s_color;

layout(push_constant) uniform PushConstantCameraData {
    mat4 proj;
    mat4 view;
};

void main() {
  // vec2 outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
  // gl_Position = vec4(outUV * 2.0 - 1.0, 0.0, 1.0);

  // pos = vec3(outUV * 2.0 - 1.0, 0.0);
  //

  gl_Position = proj * view * vec4(
    position.x + ofs.x * sin((position.x + fun_factor.x) * 0.5), 
    position.y + ofs.y * sin((position.y + fun_factor.y) * 0.5), 
    position.z + ofs.z, 
    1.0
  );
  s_color = color;
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

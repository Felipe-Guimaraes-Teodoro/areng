// adaptation of https://github.com/Tenebryo/imgui-vulkano-renderer/blob/master/src/lib.rs
// for imgui 0.11 and vulkano 0.34 
use imgui::{DrawVert, Textures, DrawCmd, DrawCmdParams, internal::RawWrapper, TextureId, ImString};

use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};

#[repr(C)]
#[derive(BufferContents, Vertex)]
struct ImVertex {
    #[format(R32G32B32_SFLOAT)]
    pub pos: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
    #[format(R32_UINT)]
    pub col: u32,
}


pub struct ImguiRenderer {
     
}

impl ImguiRenderer {

}

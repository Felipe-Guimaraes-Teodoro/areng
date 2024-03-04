use vulkano::buffer::subbuffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

use crate::rvkp::init::Vk;
use crate::rvkp::presenter::vert;

pub struct Mesh {
    pub vert_buf: Option<Subbuffer<[crate::rvkp::presenter::FVertex3d]>>,    
    pub ind_buf: Option<Subbuffer<[u32]>>,    
    pub inst_buf: Option<Subbuffer<[crate::rvkp::presenter::InstanceData]>>,

    // transform_mat: [[f32; 4];4],
}

impl Mesh {
    pub fn new() -> Self {  
        Self {
            vert_buf: None,
            ind_buf: None,
            inst_buf: None,
        }
    }

    pub fn quad(vk: &Vk) -> Self {
        let vert_buf = vk.vertex_buffer(
            vec![
                vert(0.1, 0.1, 0.0), 
                vert(0.1, -0.1, 0.0),
                vert(-0.1, 0.1, 0.0),
                vert(-0.1, -0.1, 0.0),
            ],
        );

        let ind_buf = vk.index_buffer(vec![0, 1, 2, 2, 1, 3]);

        Self {
            vert_buf: Some(vert_buf),
            ind_buf: Some(ind_buf),
            inst_buf: None,
        }
    }

    pub fn vertices(mut self, v: Vec<crate::rvkp::presenter::FVertex3d>, vk: &Vk) -> Self {
        self.vert_buf = Some(vk.vertex_buffer(v));

        self
    }

    pub fn indices(mut self, i: Vec<u32>, vk: &Vk) -> Self {
        self.ind_buf = Some(vk.index_buffer(i));

        self
    }

    pub fn instances(mut self, i: Vec<crate::rvkp::presenter::InstanceData>, vk: &Vk) -> Self {
        self.inst_buf = Some(vk.instance_buffer(i));

        self
    }


    pub fn draw(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>>) {
        let vert_buf = self.vert_buf.clone().unwrap();
        let ind_buf = self.ind_buf.clone().unwrap();
        if let Some(inst_buf) = self.inst_buf.clone() {
            builder
                .bind_vertex_buffers(0, (vert_buf.clone(), inst_buf.clone()))
                .unwrap()
                .bind_index_buffer(ind_buf.clone())
                .unwrap()
                .draw_indexed(ind_buf.len() as u32, inst_buf.len() as u32, 0, 0, 0)
                .unwrap();
        } else {
            builder
                .bind_vertex_buffers(0, vert_buf.clone())
                .unwrap()
                .bind_index_buffer(ind_buf.clone())
                .unwrap()
                .draw_indexed(ind_buf.len() as u32, 0 as u32, 0, 0, 0)
                .unwrap();
        }
    }
}

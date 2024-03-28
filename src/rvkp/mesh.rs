use std::sync::{Arc, Mutex};

use vulkano::buffer::subbuffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

use crate::rvkp::init::Vk;
use crate::rvkp::presenter::vert;
use crate::rvkp::presenter::InstanceData;

use super::vk_impl::VkImpl;
use super::vk_renderer::RVertex3d;


#[derive(Clone)]
pub struct Mesh {
    pub vert_buf: Option<Subbuffer<[RVertex3d]>>,    
    pub ind_buf: Option<Subbuffer<[u32]>>,    
    pub inst_buf: Option<Subbuffer<[crate::rvkp::presenter::InstanceData]>>,
    // transform_mat: [[f32; 4];4],
}

impl Mesh {
    pub fn new(
        verts: Vec<RVertex3d>, 
        inds: Vec<u32>,
        instcs: Vec<crate::rvkp::presenter::InstanceData>,
        vk: &VkImpl,
    ) -> Self { 
        let vert_buf = Some(vk.vertex_buffer(verts));
        let ind_buf = Some(vk.index_buffer(inds));
        //let inst_buf = Some(vk.instance_buffer(instcs));

        Self {
            vert_buf,
            ind_buf,
            inst_buf: None, // for now
        }
    }

    pub fn quad(vk: &VkImpl) -> Self {
        let vert_buf = vk.vertex_buffer(
            vec![
                RVertex3d::new(0.1, 0.1, 0.0), 
                RVertex3d::new(0.1, -0.1, 0.0),
                RVertex3d::new(-0.1, 0.1, 0.0),
                RVertex3d::new(-0.1, -0.1, 0.0),
            ],
        );

        let ind_buf = vk.index_buffer(vec![0, 1, 2, 2, 1, 3]);
        let inst_buf = None;

        Self {
            vert_buf: Some(vert_buf),
            ind_buf: Some(ind_buf),
            inst_buf,
        }
    }

    pub fn vertices(mut self, v: Vec<RVertex3d>, vk: &VkImpl) -> Self {
        self.vert_buf = Some(vk.vertex_buffer(v));

        self
    }

    pub fn indices(mut self, i: Vec<u32>, vk: &VkImpl) -> Self {
        self.ind_buf = Some(vk.index_buffer(i));

        self
    }

    // TODO!
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

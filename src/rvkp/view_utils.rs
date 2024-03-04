use crate::rvkp::{presenter::VkView, mesh::Mesh};

use vulkano::buffer::Subbuffer;

impl VkView {
    pub fn push_mesh(
        &mut self,
        vert: vulkano::buffer::Subbuffer<[crate::rvkp::presenter::FVertex3d]>,
        indx: vulkano::buffer::Subbuffer<[u32]>,
    ) {
        todo!();
    }

    pub fn set_mesh(
        &mut self,
        idx: usize,
        vert: vulkano::buffer::Subbuffer<[crate::rvkp::presenter::FVertex3d]>,
        indx: vulkano::buffer::Subbuffer<[u32]>,
    ) {
        todo!();
    }
}

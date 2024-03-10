use crate::rvkp::{presenter::VkView, mesh::Mesh};

use vulkano::buffer::Subbuffer;
use crate::rvkp::mesh::Mesh;

impl VkView {
    pub fn push_mesh(
        &mut self,
        mesh: Mesh,
    ) {
        self.meshes.push(mesh);
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

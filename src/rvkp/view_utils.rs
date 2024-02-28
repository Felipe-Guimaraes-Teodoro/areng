use crate::rvkp::presenter::VkView;

impl VkView {
    pub fn push_b_objs(
        &mut self,
        vert: vulkano::buffer::Subbuffer<[crate::rvkp::presenter::FVertex3d]>,
        indx: vulkano::buffer::Subbuffer<[u32]>,
    ) {
        self.vert_buffers.push(vert);
        self.index_buffers.push(indx);
    }

    pub fn set_b_objs(
        &mut self,
        idx: usize,
        vert: vulkano::buffer::Subbuffer<[crate::rvkp::presenter::FVertex3d]>,
        indx: vulkano::buffer::Subbuffer<[u32]>,
    ) {
        self.vert_buffers[idx] = vert;
        self.index_buffers[idx] = indx;
    }
}

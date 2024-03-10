use tokio::sync::{mpsc, Mutex};
use tokio::task;

use std::sync::{Arc, Mutex as StdMutex};

use once_cell::sync::Lazy;

use crate::rvkp::mesh::Mesh;
use crate::rvkp::init::Vk;
use crate::rvkp::presenter::VkView;

const CHUNK_SIZE: usize = 32;

pub async fn init() {
    task::spawn(async {
        loop {
            let receiver = &VOXGEN_CH.job_receiver;
            let mut rcv_guard = receiver.lock().await;

            if let Some(rcv_result) = rcv_guard.recv().await {
                let mesh = VoxelMeshGen::execute(rcv_result.0);
                let view = rcv_result.1;
                let vk = rcv_result.2;

                let mut view_guard = view.lock().unwrap();
                let vk_guard = vk.lock().unwrap();

                view_guard.meshes.clear();
                view_guard.push_mesh(
                    Mesh::quad(&vk_guard)
                );
            }
        }
    });
}

pub struct VoxelGenChannel {
    job_sender: mpsc::Sender<(VoxelMeshGenJob, Arc<StdMutex<VkView>>, Arc<StdMutex<Vk>>)>,
    job_receiver: Arc<Mutex<mpsc::Receiver<(VoxelMeshGenJob, Arc<StdMutex<VkView>>, Arc<StdMutex<Vk>>)>>>,
}

impl VoxelGenChannel {
    pub async fn send(
        &self, 
        job: VoxelMeshGenJob, 
        view: Arc<StdMutex<VkView>>, 
        vk: Arc<StdMutex<Vk>>,
    ) {
        self.job_sender.send((job, view, vk)).await.unwrap();
    }
}

pub static VOXGEN_CH: Lazy<Arc<VoxelGenChannel>> = Lazy::new(|| {
    let (job_sender, job_receiver) = mpsc::channel::<(VoxelMeshGenJob, Arc<StdMutex<VkView>>, Arc<StdMutex<Vk>>)>(100);
    let job_receiver = Arc::new(Mutex::new(job_receiver));

    let channel = VoxelGenChannel {
        job_sender,
        job_receiver,
    };

    Arc::new(channel)
});

pub struct VoxelMeshGenJob {
    voxels: Vec<bool>,
}

impl VoxelMeshGenJob {
    pub fn chunk() -> Self {
        let mut voxels = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for i in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            voxels.push(false); 
        }

        Self {
            voxels    
        }
    }
}


struct VoxelMeshGen {}

impl VoxelMeshGen {
    pub fn execute(job: VoxelMeshGenJob) {
        let voxels = job.voxels;

        print!("{:?}", voxels);
    }
}

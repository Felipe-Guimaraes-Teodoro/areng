use tokio::sync::{mpsc, Mutex};
use tokio::task;

use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::rvkp::mesh::Mesh;

const CHUNK_SIZE: usize = 32;

pub async fn init() {
    task::spawn(async {
        loop {
            let receiver = &VOXGEN_CH.job_receiver;
            let mut rcv_guard = receiver.lock().await;

            if let Some(job) = rcv_guard.recv().await {
                VoxelMeshGen::execute(job);
            }
        }
    });
}

pub struct VoxelGenChannel {
    job_sender: mpsc::Sender<VoxelMeshGenJob>,
    job_receiver: Arc<Mutex<mpsc::Receiver<VoxelMeshGenJob>>>,


    mesh_sender: mpsc::Sender<Mesh>,
    mesh_receiver: Arc<Mutex<mpsc::Receiver<Mesh>>>,
}

impl VoxelGenChannel {
    pub async fn send(&self, job: VoxelMeshGenJob) {
        self.job_sender.send(job).await.unwrap();
    }

    pub async fn get(&self, job: VoxelMeshGenJob) -> Option<Mesh> {
        let receiver = &VOXGEN_CH.mesh_receiver;
        let mut rcv_guard = receiver.lock().await;

        return rcv_guard.recv().await 
    }
}

pub static VOXGEN_CH: Lazy<Arc<VoxelGenChannel>> = Lazy::new(|| {
    let (job_sender, job_receiver) = mpsc::channel::<VoxelMeshGenJob>(100);
    let job_receiver = Arc::new(Mutex::new(job_receiver));


    let (mesh_sender, mesh_receiver) = mpsc::channel::<Mesh>(100);
    let mesh_receiver = Arc::new(Mutex::new(mesh_receiver));

    let channel = VoxelGenChannel {
        job_sender,
        job_receiver,
        mesh_sender,
        mesh_receiver,
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

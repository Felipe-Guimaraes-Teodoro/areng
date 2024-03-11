use tokio::sync::{mpsc, Mutex};
use tokio::task;

use std::sync::{Arc, Mutex as StdMutex};

use once_cell::sync::Lazy;

use crate::rvkp::mesh::Mesh;
use crate::rvkp::init::Vk;
use crate::rvkp::presenter::VkView;

use crate::utils::{idx_to_vec3, vec3_to_idx, random};

const CHUNK_SIZE: usize = 32;
const WORLD_SIZE: usize = 32;

const CHUNKS: Lazy<Arc<StdMutex<Vec<bool>>>> = Lazy::new(|| {
    let mut chunks = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
    for _i in 0..WORLD_SIZE * WORLD_SIZE * WORLD_SIZE {
        chunks.push(false);
    }

    Arc::new(StdMutex::new(chunks))
});

pub async fn init() {
    task::spawn(async {
        loop {
            let receiver = &VOXGEN_CH.job_receiver;
            let mut rcv_guard = receiver.lock().await;

            if let Some(rcv_result) = rcv_guard.recv().await {
                let view = rcv_result.1;
                let mut view_guard = view.lock().unwrap();

                let vk = rcv_result.2;
                let vk_guard = vk.lock().unwrap();

                let mesh = VoxelMeshGen::execute(rcv_result.0, &vk_guard);

                // view_guard.meshes.clear();
                view_guard.push_mesh(
                    mesh
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

use glam::{Vec3A, vec3a};
pub struct VoxelMeshGenJob {
    voxels: Vec<bool>,
    pos: Vec3A,
}

impl VoxelMeshGenJob {
    pub fn chunk(x: f32, y: f32, z: f32) -> Option<Self> {
        let chunk_idx = vec3_to_idx(x as usize, y as usize, z as usize, CHUNK_SIZE);
        let mut voxels = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);

        if CHUNKS.lock().unwrap()[chunk_idx] == true { 
            return None 
        }

        for _i in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            voxels.push(false); 
        }
        CHUNKS.lock().unwrap()[chunk_idx] = true;

        Some(Self {
            voxels,
            pos: vec3a(x * 32.0, y * 32.0, z * 32.0),
        })
    }
}


struct VoxelMeshGen {}

use crate::rvkp::presenter::{vert, FVertex3d, InstanceData};
use threadpool::ThreadPool;
impl VoxelMeshGen {
    pub fn execute(job: VoxelMeshGenJob, vk: &Vk) -> Mesh {
        let voxels = job.voxels;
        let pool = ThreadPool::new(12);

        let mut verts = vec![];
        let mut inds = vec![];
        let instcs = vec![InstanceData {
            ofs: [0.0, 0.0, 0.0], fun_factor: [0.0, 0.0, 0.0] 
        }];

        assert_eq!(voxels.len(), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);

        let chunk_size = CHUNK_SIZE as f32;

        for i in 0..voxels.len() {
            let voxel_coord = idx_to_vec3(i, CHUNK_SIZE) + job.pos * chunk_size;
            verts.push(vert(voxel_coord.x + random(0.0, 1.0), voxel_coord.y + random(0.0, 1.0), voxel_coord.z + random(0.0, 1.0)));
            inds.push(i as u32);
        }

        let mesh = Mesh::new(verts, inds, instcs, &vk);

        mesh
    }
}

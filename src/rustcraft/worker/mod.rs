
use std::sync::{Arc, mpsc::*};
use crate::prelude::*;

pub struct WorkerData {
    pub registry: Arc<ItemRegistry>,
}

pub struct JobDispatcher {
    tx: Sender<WorkerJob>,
    rx: Receiver<WorkerResponse>,
}

impl JobDispatcher {

    pub fn iter_responses(&mut self) -> TryIter<'_, WorkerResponse> {
        self.rx.try_iter()
    }

    pub fn new(wdata: WorkerData) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (dtx, drx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            worker_thread(rx, dtx, wdata)
        });
        JobDispatcher {
            tx,
            rx: drx
        }
    }
    pub fn send(&self, work: WorkerJob) {
        self.tx.send(work);
    }
}

pub enum WorkerJob {
    SaveChunk(Box<Chunk>),
    LoadChunk(i32,i32,i32),
}
pub enum WorkerResponse {
    LoadedChunk(Option<Box<Chunk>>),
}
fn worker_thread(rx: Receiver<WorkerJob>, tx: Sender<WorkerResponse>, data: WorkerData) {
    'work: loop {
        let job = match rx.recv() {
            Err(_) => {break 'work}
            Ok(job) => job
        };
        use WorkerJob::*;
        /* match job {
            SaveChunk(chunk) => {
                std::fs::write(format!("save/{:x}_{:x}_{:x}.chunk", chunk.pos.x, chunk.pos.y, chunk.pos.z), chunk.save());
            },
            LoadChunk(x,y,z) => {
                tx.send(
                    WorkerResponse::LoadedChunk(
                        Chunk::load(x, y, z, data.registry.as_ref()).map(Box::new)
                    )
                ).unwrap();
            }
        }; */
    }
}
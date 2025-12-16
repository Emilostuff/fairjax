use fairjax::*;
use fairjax_core::MailBox;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum JobKind {
    Render,
    ComputeValue,
    Encrypt,
    Decrypt,
    Infer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResourceType {
    Cpu { cores: usize, ram_gb: usize },
    Gpu { vram_gb: usize },
    NeuralEngine,
}

impl ResourceType {
    pub fn cpu(cores: usize, ram_gb: usize) -> Self {
        ResourceType::Cpu { cores, ram_gb }
    }

    pub fn gpu(vram_gb: usize) -> Self {
        ResourceType::Gpu { vram_gb }
    }

    pub fn neural_engine() -> Self {
        ResourceType::NeuralEngine {}
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    Job {
        job_id: usize,
        kind: JobKind,
        priority: u8,
    },
    WorkerAvailable {
        worker_id: usize,
        resource: ResourceType,
    },
}

impl Msg {
    pub fn job(job_id: usize, kind: JobKind, priority: u8) -> Self {
        Msg::Job {
            job_id,
            kind,
            priority,
        }
    }

    pub fn worker(worker_id: usize, resource: ResourceType) -> Self {
        Msg::WorkerAvailable {
            worker_id,
            resource,
        }
    }
}

fn run_job(job_id: usize, worker_ids: Vec<usize>) {
    println!("Running job {} using workers {:?}", job_id, worker_ids);
}

use JobKind::*;
use Msg::*;
use ResourceType::*;

fn main() {
    let mut mailbox = MailBox::default();
    let messages = vec![
        Msg::job(1, Render, 12),
        Msg::job(2, Infer, 100),
        Msg::job(3, Infer, 100),
        Msg::job(4, Encrypt, 5),
        Msg::worker(204, ResourceType::gpu(11)),
        Msg::worker(209, ResourceType::cpu(192, 2048)),
        Msg::worker(201, ResourceType::cpu(8, 4)),
        Msg::worker(269, ResourceType::cpu(16, 64)),
        Msg::worker(217, ResourceType::neural_engine()),
    ];

    for msg in messages {
        #[rustfmt::skip]
        fairjax!(match msg >> [mailbox, Msg] {
            (
                Job { job_id, kind: Render, .. },
                WorkerAvailable {
                    worker_id,
                    resource: Gpu { vram_gb: 8..=24 },
                },
            ) => run_job(job_id, vec![worker_id]),
            (
                Job { job_id, kind: ComputeValue | Encrypt | Decrypt, .. },
                WorkerAvailable { worker_id, resource: Cpu { .. } },
            ) => run_job(job_id, vec![worker_id]),
            (
                Job { job_id, kind: Infer, priority: 50.. },
                WorkerAvailable {
                    worker_id,
                    resource: NeuralEngine,
                },
            )  => run_job(job_id, vec![worker_id]),
            (
                Job { job_id, kind: Infer, .. },
                WorkerAvailable {
                    worker_id,
                    resource: Cpu { cores: 64.., ram_gb: 128.. },
                },
            ) => run_job(job_id, vec![worker_id]),
        });
    }
}

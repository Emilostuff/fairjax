use actix::prelude::*;
use fairjax::*;
use fairjax_core::MailBox;
use rand::Rng;
use std::collections::HashMap;
use std::io::{self, Write};
use std::{thread::sleep, time::Duration};

use Event::*;
use JobKind::*;
use Resource::*;
use Status::*;

/// Simulation of multiple actors working together:
/// - 1 Coordinator to match jobs to available workers
/// - 1 Monitor to keep track of progress and log to the terminal
/// - Multiple workers with different capacities (see below)
#[actix::main]
async fn main() {
    let m = Monitor::start_default();
    let c = Coordinator::new(m.clone()).start();

    // Wait for coordinator and monitor to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Define worker specs
    let workers = [
        ("CPU small 1", 0, Resource::cpu(4, 8)),
        ("CPU small 2", 0, Resource::cpu(8, 8)),
        ("CPU small 3", 0, Resource::cpu(8, 8)),
        ("CPU medium 1", 30, Resource::cpu(16, 32)),
        ("CPU medium 2", 30, Resource::cpu(16, 48)),
        ("CPU premium 1", 60, Resource::cpu(32, 256)),
        ("CPU premium 2", 60, Resource::cpu(48, 256)),
        ("Neural Engine", 50, Resource::neural_engine()),
        ("GPU small", 0, Resource::gpu(11)),
        ("GPU premium", 60, Resource::gpu(32)),
    ];

    // Create and register workers in coordinator
    for (id, min_priority, resource) in workers.into_iter() {
        let worker = Worker::new(id, resource, min_priority, m.clone(), c.clone());
        let addr = SyncArbiter::start(1, move || worker.clone());
        c.do_send(Event::register(id, addr));
    }

    // Wait for workers to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Create and push random jobs to coordinator
    let mut rng = rand::rng();
    for i in 0..100 {
        let kind = match rng.random_range(0..=4) {
            0 => Render,
            1 => ComputeValue,
            2 => Encrypt,
            3 => Decrypt,
            _ => Infer,
        };
        c.do_send(Event::job(i, kind, rng.random_range(1..=100)));
        tokio::time::sleep(Duration::from_millis(rng.random_range(20..100))).await;
    }

    tokio::signal::ctrl_c().await.unwrap();
}

// ACTORS /////////////////////////////////////////////////////////////

/// Coordinator Actor
struct Coordinator {
    mailbox: MailBox<Event>,
    monitor: Addr<Monitor>,
    workers: HashMap<&'static str, Addr<Worker>>,
}

impl Actor for Coordinator {
    type Context = Context<Self>;
}

impl Coordinator {
    fn new(monitor: Addr<Monitor>) -> Self {
        Self {
            mailbox: MailBox::default(),
            monitor: monitor,
            workers: HashMap::new(),
        }
    }
}

impl Handler<Event> for Coordinator {
    type Result = ();

    fn handle(&mut self, msg: Event, _: &mut Context<Self>) {
        // Emit status messages on receive
        match &msg {
            JobRequest {
                job: Job { id, .. },
                ..
            } => self.monitor.do_send(JobQueued { job_id: *id }),
            WorkerAvailable { worker_id, .. } => self.monitor.do_send(Ready { worker_id }),
            _ => (),
        };

        // Consume and match with other received messages
        #[rustfmt::skip]
        fairjax!(match msg >> [self.mailbox, Event] {
            // Register a new worker actor in coordinator for future use
            RegisterWorkerAddress{ worker_id, worker_addr } => {
                self.workers.insert(worker_id, worker_addr);
            }
            // Match GPU resource to any render job
            (
                JobRequest { job: job @ Job { id, kind: Render }, .. },
                WorkerAvailable { worker_id, resource: Gpu { vram_gb: 8.. }, .. },
            ) => self.workers[worker_id].do_send(job),
            // Match CPU resource to CPU bound job with sufficiently high priority
            (
                JobRequest { job: job @ Job { id, kind: ComputeValue | Encrypt | Decrypt }, priority },
                WorkerAvailable { worker_id, min_priority, resource: Cpu { .. } },
            ) if priority >= min_priority => self.workers[worker_id].do_send(job),
            // Match a Neural Engine ressource to a inference job with sufficiently high priority (if possible)
            (
                JobRequest { job: job @ Job { id, kind: Infer }, priority },
                WorkerAvailable { worker_id, resource: NeuralEngine { .. }, min_priority },
            ) if priority >= min_priority => self.workers[worker_id].do_send(job),
            // Match a capable CPU ressource to any inference job regardless of priority
            (
                JobRequest { job: job @ Job { id, kind: Infer }, .. },
                WorkerAvailable { worker_id, resource: Cpu { cores: 8..24, ram_gb: 32.. }, ..  },
            ) => self.workers[worker_id].do_send(job),
        });
    }
}

/// Worker Actor
#[derive(Clone)]
struct Worker {
    id: &'static str,
    resource: Resource,
    min_priority: u8,
    monitor: Addr<Monitor>,
    coordinator: Addr<Coordinator>,
}

impl Worker {
    fn new(
        id: &'static str,
        resource: Resource,
        min_priority: u8,
        monitor: Addr<Monitor>,
        coordinator: Addr<Coordinator>,
    ) -> Self {
        Self {
            id,
            resource,
            min_priority,
            monitor,
            coordinator,
        }
    }

    fn send_available(&self) {
        self.coordinator.do_send(WorkerAvailable {
            worker_id: self.id,
            min_priority: self.min_priority,
            resource: self.resource.clone(),
        });
    }
}

impl Actor for Worker {
    type Context = SyncContext<Self>;

    // Fenceposting - ensure Worker is registered as available after startup
    fn started(&mut self, _ctx: &mut Self::Context) {
        self.send_available();
    }
}

impl Handler<Job> for Worker {
    type Result = ();

    fn handle(&mut self, Job { id, .. }: Job, _ctx: &mut SyncContext<Self>) {
        // Update worker status in Monitor
        self.monitor.do_send(Processing {
            worker_id: self.id,
            job_id: id,
        });

        // Simulate blocking job
        sleep(Duration::from_millis(rand::rng().random_range(500..1000)));

        // Register as available again once job is done
        self.send_available();
    }
}

// Monitor Actor
#[derive(Default)]
struct Monitor {
    workers: Vec<(&'static str, Option<usize>)>,
    jobs: Vec<usize>,
}

impl Actor for Monitor {
    type Context = Context<Self>;
}

impl Handler<Status> for Monitor {
    type Result = ();

    fn handle(&mut self, msg: Status, _: &mut Context<Self>) {
        // Update state of Worker or Job queue
        match msg {
            Processing { worker_id, job_id } => {
                self.jobs.retain(|&id| id != job_id);
                let _ = self
                    .workers
                    .iter_mut()
                    .find(|e| e.0 == worker_id)
                    .map(|(_, job)| *job = Some(job_id));
            }
            Ready { worker_id } => {
                if let Some((_, job)) = self.workers.iter_mut().find(|e| e.0 == worker_id) {
                    *job = None;
                } else {
                    self.workers.push((worker_id, None));
                }
            }
            JobQueued { job_id } => self.jobs.push(job_id),
        };

        // Clear Terminal
        print!("\x1B[2J\x1B[1;1H");

        // Print state of each worker
        for (id, job) in self.workers.iter() {
            match job {
                Some(job_id) => println!("{}:\t\x1B[34mProcessing: {:?}\x1B[0m", id, job_id),
                None => println!("{}:\t\x1B[32mReady\x1B[0m", id),
            }
        }

        // Print Job Queue and exit message + flush
        println!("\nJobs:\n{:?}", self.jobs);
        println!("\n\x1B[33mPress Ctrl+C to shutdown system\x1B[0m");
        io::stdout().flush().unwrap();
    }
}

// MESSAGE TYPES /////////////////////////////////////////////////////////

/// Coordinator Message
#[derive(Clone, Debug, PartialEq)]
enum Event {
    JobRequest {
        job: Job,
        priority: u8,
    },
    WorkerAvailable {
        worker_id: &'static str,
        resource: Resource,
        min_priority: u8,
    },
    RegisterWorkerAddress {
        worker_id: &'static str,
        worker_addr: Addr<Worker>,
    },
}

impl Message for Event {
    type Result = ();
}

impl Event {
    pub fn job(id: usize, kind: JobKind, priority: u8) -> Self {
        Event::JobRequest {
            job: Job { id, kind },
            priority,
        }
    }

    pub fn register(worker_id: &'static str, worker_addr: Addr<Worker>) -> Self {
        Event::RegisterWorkerAddress {
            worker_id,
            worker_addr,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Resource {
    Cpu { cores: usize, ram_gb: usize },
    Gpu { vram_gb: usize },
    NeuralEngine,
}

impl Resource {
    pub fn cpu(cores: usize, ram_gb: usize) -> Self {
        Resource::Cpu { cores, ram_gb }
    }

    pub fn gpu(vram_gb: usize) -> Self {
        Resource::Gpu { vram_gb }
    }

    pub fn neural_engine() -> Self {
        Resource::NeuralEngine {}
    }
}

/// Status Message
pub enum Status {
    Processing {
        worker_id: &'static str,
        job_id: usize,
    },
    Ready {
        worker_id: &'static str,
    },
    JobQueued {
        job_id: usize,
    },
}

impl Message for Status {
    type Result = ();
}

/// Job Message
#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub id: usize,
    pub kind: JobKind,
}

impl Message for Job {
    type Result = ();
}

impl Job {
    pub fn new(id: usize, kind: JobKind) -> Self {
        Job { id, kind }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum JobKind {
    Render,
    ComputeValue,
    Encrypt,
    Decrypt,
    Infer,
}

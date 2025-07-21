use core::time::Duration;

use async_graphql::{Context, Object, Schema, SimpleObject};
use sysinfo::{Pid, Process, System};

#[derive(SimpleObject, Clone)]
pub struct ProcessInfo {
    pub pid: i32,
    pub usage: f32,
    pub name: String,
    pub rss: u64,
    pub runtime_ms: u64,
    pub vsz: u64,
}

impl ProcessInfo {
    pub fn from_process(process: &Process) -> Self {
        Self {
            pid: process.pid().as_u32() as i32,
            usage: process.cpu_usage(),
            name: process.name().to_string_lossy().into(),
            rss: process.memory(),
            runtime_ms: process.run_time() * 1000,
            vsz: process.virtual_memory(),
        }
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn processes(
        &self,
        _ctx: &Context<'_>,
        pid: Option<i32>,
        min_usage: Option<f32>,
        min_rss_kb: Option<u64>,
        min_runtime_ms: Option<u64>,
        sleep_ms: Option<u64>,
    ) -> Vec<ProcessInfo> {
        let mut system = System::new_all();
        system.refresh_all();
        let sleep_duration = sleep_ms.unwrap_or(500);
        std::thread::sleep(Duration::from_millis(sleep_duration));
        system.refresh_all();

        let mut processes = if let Some(pid_val) = pid {
            if let Some(process) = system.process(Pid::from(pid_val as usize)) {
                vec![ProcessInfo::from_process(process)]
            } else {
                vec![]
            }
        } else {
            system
                .processes()
                .values()
                .map(ProcessInfo::from_process)
                .collect()
        };

        if let Some(min_usage_val) = min_usage {
            processes.retain(|p| p.usage > min_usage_val);
        }

        if let Some(min_rss_val) = min_rss_kb {
            processes.retain(|p| p.rss > min_rss_val * 1024);
        }

        if let Some(min_runtime_val) = min_runtime_ms {
            processes.retain(|p| p.runtime_ms > min_runtime_val);
        }

        processes
    }
}

pub type ProcessSchema =
    Schema<Query, async_graphql::EmptyMutation, async_graphql::EmptySubscription>;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreUsage {
    pub name: String,
    pub usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub used: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub name: String,
    pub mount_point: String,
    pub used: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUsage {
    pub received: u64,
    pub transmitted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_total: f32,
    pub cpu_cores: Vec<CpuCoreUsage>,
    pub memory: MemoryUsage,
    pub disks: Vec<DiskUsage>,
    pub network: NetworkUsage,
}


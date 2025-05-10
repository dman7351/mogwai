//! System Hardware Information Reporter
//!
//! This program gathers basic hardware information from Windows, macOS, and Linux systems
//! and generates a standardized report suitable for AI input or system analysis.
//! 
//! running: cargo run --bin sys_info | python3 ./src/bin/mogAI.py | ./src/bin/deploy_files.sh

use chrono::prelude::*;
use hostname::get as get_hostname;
use os_info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use sysinfo::{System, RefreshKind};

#[derive(Serialize, Deserialize, Debug)]
struct CpuInfo {
    model: String,
    physical_cores: Option<usize>,
    total_cores: usize,
    max_frequency: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MemoryInfo {
    total: String,
    available: String,
    used_percent: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct DiskInfo {
    device: String,
    mountpoint: Option<String>,
    filesystem: Option<String>,
    total: String,
    free: String,
    used_percent: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct NetworkInfo {
    name: String,
    mac_address: Option<String>,
    ip_addresses: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SystemBasicInfo {
    name: String,
    version: String,
    platform: String,
    machine: String,
    hostname: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Dependencies {
    sysinfo: bool,
    wmi: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct SystemInfo {
    system: SystemBasicInfo,
    timestamp: String,
    cpu: Option<CpuInfo>,
    memory: Option<MemoryInfo>,
    disks: Option<Vec<DiskInfo>>,
    network: Option<Vec<NetworkInfo>>,
    dependencies: Dependencies,
    error: Option<String>,
}

/// Format bytes to human-readable format
fn get_size_format(bytes: u64, factor: u64, suffix: &str) -> String {
    let units = ["", "K", "M", "G", "T", "P", "Y"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= factor as f64 && unit_index < units.len() - 1 {
        size /= factor as f64;
        unit_index += 1;
    }

    format!("{:.2} {}{}", size, units[unit_index], suffix)
}

/// Gather information for Windows systems
#[cfg(target_os = "windows")]
fn get_windows_info(sys: &System) -> HashMap<String, serde_json::Value> {
    let mut info = HashMap::new();

    // CPU Information
    let cpu_info = CpuInfo {
        model: sys.cpus().get(0).map(|cpu| cpu.brand().to_string()).unwrap_or_else(|| "unknown".to_string()),
        physical_cores: sys.physical_core_count(),
        total_cores: sys.cpus().len(),
        max_frequency: sys.cpus().get(0).map(|cpu| format!("{} MHz", cpu.frequency())),
    };

    info.insert("cpu".to_string(), serde_json::to_value(cpu_info).unwrap());

    // Memory Information
    let memory_info = MemoryInfo {
        total: get_size_format(sys.total_memory(), 1024, "B"),
        available: get_size_format(sys.available_memory(), 1024, "B"),
        used_percent: ((sys.total_memory() - sys.available_memory()) as f32 / sys.total_memory() as f32) * 100.0,
    };
    info.insert("memory".to_string(), serde_json::to_value(memory_info).unwrap());

    // Disk Information
    let mut disks = Vec::new();
    for disk in sys.disks() {
        disks.push(DiskInfo {
            device: disk.name().to_str().unwrap_or("Unknown").to_string(),
            mountpoint: Some(disk.mount_point().to_str().unwrap_or("Unknown").to_string()),
            filesystem: None,
            total: get_size_format(disk.total_space(), 1024, "B"),
            free: get_size_format(disk.available_space(), 1024, "B"),
            used_percent: ((disk.total_space() - disk.available_space()) as f32 / disk.total_space() as f32) * 100.0,
        });
    }
    info.insert("disks".to_string(), serde_json::to_value(disks).unwrap());

    // Network Information
    let mut networks = Vec::new();
    for (interface_name, _network) in sys.networks().iter() {
        networks.push(NetworkInfo {
            name: interface_name.to_string(),
            mac_address: None, // sysinfo doesn't provide MAC addresses directly
            ip_addresses: vec![],
        });
    }
    info.insert("network".to_string(), serde_json::to_value(networks).unwrap());

    info
}

/// Gather information for Linux systems
#[cfg(target_os = "linux")]
fn get_linux_info(sys: &System) -> HashMap<String, serde_json::Value> {
    let mut info = HashMap::new();

    // CPU Information
    let mut cpu_info = CpuInfo {
        model: sys.cpus().get(0).map(|cpu| cpu.brand().to_string()).unwrap_or_else(|| "unknown".to_string()),
        physical_cores: sys.physical_core_count(),
        total_cores: sys.cpus().len(),
        max_frequency: None,
    };

    // Try to get more detailed CPU info from /proc/cpuinfo
    if let Ok(output) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in output.lines() {
            if line.starts_with("model name") {
                if let Some(model) = line.split(':').nth(1) {
                    cpu_info.model = model.trim().to_string();
                    break;
                }
            }
        }
    }

    info.insert("cpu".to_string(), serde_json::to_value(cpu_info).unwrap());

    // Memory Information
    let memory_info = MemoryInfo {
        total: get_size_format(sys.total_memory(), 1024, "B"),
        available: get_size_format(sys.available_memory(), 1024, "B"),
        used_percent: ((sys.total_memory() - sys.available_memory()) as f32 / sys.total_memory() as f32) * 100.0,
    };
    info.insert("memory".to_string(), serde_json::to_value(memory_info).unwrap());

    // Disk Information
    let mut disks = Vec::new();
    for disk in sys.disks() {
        disks.push(DiskInfo {
            device: disk.name().to_str().unwrap_or("Unknown").to_string(),
            mountpoint: Some(disk.mount_point().to_str().unwrap_or("Unknown").to_string()),
            filesystem: Some(disk.file_system().to_string_lossy().into_owned()),
            total: get_size_format(disk.total_space(), 1024, "B"),
            free: get_size_format(disk.available_space(), 1024, "B"),
            used_percent: ((disk.total_space() - disk.available_space()) as f32 / disk.total_space() as f32) * 100.0,
        });
    }
    info.insert("disks".to_string(), serde_json::to_value(disks).unwrap());

    // Network Information
    let mut networks = Vec::new();
    for (interface_name, _network) in sys.networks().iter() {
        networks.push(NetworkInfo {
            name: interface_name.to_string(),
            mac_address: None,
            ip_addresses: vec![],
        });
    }
    info.insert("network".to_string(), serde_json::to_value(networks).unwrap());

    info
}

/// Gather information for macOS systems
#[cfg(target_os = "macos")]
fn get_macos_info(sys: &System) -> HashMap<String, serde_json::Value> {
    let mut info = HashMap::new();

    // CPU Information
    let cpu_info = CpuInfo {
        model: sys.cpus().get(0).map(|cpu| cpu.brand().to_string()).unwrap_or_else(|| "unknown".to_string()),
        physical_cores: sys.physical_core_count(),
        total_cores: sys.cpus().len(),
        max_frequency: None,
    };

    info.insert("cpu".to_string(), serde_json::to_value(cpu_info).unwrap());

    // Memory Information
    let memory_info = MemoryInfo {
        total: get_size_format(sys.total_memory(), 1024, "B"),
        available: get_size_format(sys.available_memory(), 1024, "B"),
        used_percent: ((sys.total_memory() - sys.available_memory()) as f32 / sys.total_memory() as f32) * 100.0,
    };
    info.insert("memory".to_string(), serde_json::to_value(memory_info).unwrap());

    // Disk Information
    let mut disks = Vec::new();
    for disk in (sysinfo::Disks::new_with_refreshed_list()).list() {
        disks.push(DiskInfo {
            device: disk.name().to_str().unwrap_or("Unknown").to_string(),
            mountpoint: Some(disk.mount_point().to_str().unwrap_or("Unknown").to_string()),
            filesystem: Some(disk.file_system().to_string_lossy().into_owned()),
            total: get_size_format(disk.total_space(), 1024, "B"),
            free: get_size_format(disk.available_space(), 1024, "B"),
            used_percent: ((disk.total_space() - disk.available_space()) as f32 / disk.total_space() as f32) * 100.0,
        });
    }
    info.insert("disks".to_string(), serde_json::to_value(disks).unwrap());

    // Network Information
    let mut networks = Vec::new();
    for (interface_name, _network) in &(sysinfo::Networks::new_with_refreshed_list()) {
        networks.push(NetworkInfo {
            name: interface_name.to_string(),
            mac_address: None,
            ip_addresses: vec![],
        });
    }
    info.insert("network".to_string(), serde_json::to_value(networks).unwrap());

    info
}

/// Gather all system information
fn gather_system_info() -> SystemInfo {
    // Use RefreshKind::everything() to update all available data
    let refresh_kind = RefreshKind::everything();
    let sys = System::new_with_specifics(refresh_kind);

    // Get hostname
    let hostname = match get_hostname() {
        Ok(name) => name.to_string_lossy().into_owned(),
        Err(_) => "unknown".to_string(),
    };

    // Get OS information
    let os = os_info::get();

    // Common information across all platforms
    let mut info = SystemInfo {
        system: SystemBasicInfo {
            name: os.os_type().to_string(),
            version: os.version().to_string(),
            platform: format!("{} {}", os.os_type(), os.version()),
            machine: env::consts::ARCH.to_string(),
            hostname,
        },
        timestamp: Utc::now().to_rfc3339(),
        cpu: None,
        memory: None,
        disks: None,
        network: None,
        dependencies: Dependencies {
            sysinfo: true,
            wmi: cfg!(target_os = "windows"),
        },
        error: None,
    };

    // Get OS-specific information
    let hw_info = if cfg!(target_os = "windows") {
        #[cfg(target_os = "windows")]
        {
            get_windows_info(&sys)
        }
        #[cfg(not(target_os = "windows"))]
        {
            HashMap::new()
        }
    } else if cfg!(target_os = "linux") {
        #[cfg(target_os = "linux")]
        {
            get_linux_info(&sys)
        }
        #[cfg(not(target_os = "linux"))]
        {
            HashMap::new()
        }
    } else if cfg!(target_os = "macos") {
        #[cfg(target_os = "macos")]
        {
            get_macos_info(&sys)
        }
        #[cfg(not(target_os = "macos"))]
        {
            HashMap::new()
        }
    } else {
        info.error = Some(format!("Unsupported operating system: {}", info.system.name));
        HashMap::new()
    };

    // Combine information
    if let Some(cpu) = hw_info.get("cpu") {
        info.cpu = serde_json::from_value(cpu.clone()).ok();
    }
    if let Some(memory) = hw_info.get("memory") {
        info.memory = serde_json::from_value(memory.clone()).ok();
    }
    if let Some(disks) = hw_info.get("disks") {
        info.disks = serde_json::from_value(disks.clone()).ok();
    }
    if let Some(network) = hw_info.get("network") {
        info.network = serde_json::from_value(network.clone()).ok();
    }

    info
}

/// Print a formatted report to the console
fn print_console_report(info: &SystemInfo) {
    println!("\n=== System Hardware Information Report ===");
    println!("Operating System: {} {}", info.system.name, info.system.version);
    println!("Machine Type: {}", info.system.machine);
    println!("Hostname: {}", info.system.hostname);

    if let Some(cpu) = &info.cpu {
        println!("\n--- CPU Information ---");
        println!("Model: {}", cpu.model);
        if let Some(cores) = cpu.physical_cores {
            println!("Physical Cores: {}", cores);
        }
        println!("Total Cores: {}", cpu.total_cores);
        if let Some(freq) = &cpu.max_frequency {
            println!("Max Frequency: {}", freq);
        }
    }

    if let Some(memory) = &info.memory {
        println!("\n--- Memory Information ---");
        println!("Total: {}", memory.total);
        println!("Available: {}", memory.available);
        println!("Used Percent: {:.1}%", memory.used_percent);
    }

    if let Some(disks) = &info.disks {
        println!("\n--- Disk Information ---");
        for (i, disk) in disks.iter().enumerate() {
            println!("\nDisk {}:", i + 1);
            println!("  Device: {}", disk.device);
            if let Some(mountpoint) = &disk.mountpoint {
                println!("  Mountpoint: {}", mountpoint);
            }
            if let Some(filesystem) = &disk.filesystem {
                println!("  Filesystem: {}", filesystem);
            }
            println!("  Total: {}", disk.total);
            println!("  Free: {}", disk.free);
            println!("  Used Percent: {:.1}%", disk.used_percent);
        }
    }

    if let Some(networks) = &info.network {
        println!("\n--- Network Information ---");
        for (i, nic) in networks.iter().enumerate() {
            println!("\nNetwork Interface {}:", i + 1);
            println!("  Name: {}", nic.name);
            if let Some(mac) = &nic.mac_address {
                println!("  Mac Address: {}", mac);
            }
            if !nic.ip_addresses.is_empty() {
                println!("  IP Addresses: {}", nic.ip_addresses.join(", "));
            } else {
                println!("  IP Addresses: None");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Gather all system information
    let info = gather_system_info();

    // Check if we should output human-readable format
    let args: Vec<String> = env::args().collect();
    let human_readable = args.len() > 1 && args[1] == "human";
    
    // Check if we should save to file
    let save_file = args.len() > 1 && args[1] == "save";

    if human_readable {
        // Print human-readable report
        print_console_report(&info);
    } else if save_file {
        // Print human-readable report
        print_console_report(&info);
        
        // Save to JSON file
        let filename = format!(
            "system_info_{}_{}.json",
            info.system.hostname,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        let mut file = File::create(&filename)?;
        file.write_all(serde_json::to_string_pretty(&info)?.as_bytes())?;

        println!("\nDetailed report saved to {}", filename);
    } else {
        // Default: Output JSON for piping
        println!("{}", serde_json::to_string(&info)?);
    }

    Ok(())
}
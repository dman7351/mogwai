use std::time::{Duration, Instant};
use std::thread;
use sysinfo::{System};
use std::process;

pub fn stress_memory(mb: usize, duration: u64) {
    println!("Allocating {} MB of memory...", mb);

    let mut memory_block = vec![0u8; mb * 1024 * 1024];

    println!("Memory allocated. Keeping it active for {} seconds...", duration);

    let start = Instant::now();

    // If duration is 0, run indefinitely
    if duration == 0 {
        println!("Running memory stress test indefinitely. To stop, use: kill {}", process::id());
    }

    while start.elapsed() < Duration::from_secs(duration) || duration == 0 {
        for i in (0..memory_block.len()).step_by(4096) {
            memory_block[i] = i as u8; 
        }
        thread::sleep(Duration::from_millis(500)); // Optional sleep to avoid max CPU usage

        if duration == 0 {
            continue; // Keep looping indefinitely if duration is 0
        }
    }

    println!("Memory stress test completed.");
}

pub fn check_memory_usage() {
    let mut sys = System::new_all();
    sys.refresh_memory();

    println!("Total Memory: {} MB", sys.total_memory() / 1024);
    println!("Used Memory: {} MB", sys.used_memory() / 1024);
}

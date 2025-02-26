use std::time::{Duration, Instant};
use std::thread;
use sysinfo::{System};

pub fn stress_memory(mb: usize, duration: u64) {
    println!("Allocating {} MB of memory...", mb);

    let mut memory_block = vec![0u8; mb * 1024 * 1024];

    println!("Memory allocated. Keeping it active for {} seconds...", duration);

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(duration) {
        for i in (0..memory_block.len()).step_by(4096) {
            memory_block[i] = i as u8; 
        }
        thread::sleep(Duration::from_millis(500)); 
    }

    println!("Memory stress test completed.");
}

pub fn check_memory_usage() {
    let mut sys = System::new_all();
    sys.refresh_memory();

    println!("Total Memory: {} MB", sys.total_memory() / 1024);
    println!("Used Memory: {} MB", sys.used_memory() / 1024);
}

use std::time::{Duration, Instant};
use std::thread::sleep;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use sysinfo::System;
use tokio::task;

pub async fn stress_memory(
    threads: usize,
    mb_per_thread: usize,
    duration: u64,
    stop_flag: Arc<AtomicBool>,
    task_id: String,
) {
    if duration == 0 {
        println!("Running memory stress test indefinitely. To stop, send a POST request to: http://localhost:8080/stop/{}", task_id);
    }
    
    println!(
        "Spawning {} threads. Each will allocate {} MB (Total: {} MB)",
        threads,
        mb_per_thread,
        threads * mb_per_thread
    );



    let mut handles = Vec::new();

    for thread_id in 0..threads {
        let stop = Arc::clone(&stop_flag);

        let handle = task::spawn_blocking(move || {
            let mut memory_block = vec![0u8; mb_per_thread * 1024 * 1024];
            let start = Instant::now();
            
            // if duration == 0 run indefinetly
            while (duration == 0 || start.elapsed() < Duration::from_secs(duration))
                && !stop.load(Ordering::SeqCst)
            {
                for i in (0..memory_block.len()).step_by(4096) {
                    memory_block[i] = i as u8;
                }

                // Sleep to reduce CPU 
                sleep(Duration::from_millis(500));
            }

            println!("[Thread {}] Memory stress test completed.", thread_id);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

}

pub fn check_memory_usage() {
    let mut sys = System::new_all();
    sys.refresh_memory();

    println!("Total Memory: {} MB", sys.total_memory() / 1024);
    println!("Used Memory: {} MB", sys.used_memory() / 1024);
}

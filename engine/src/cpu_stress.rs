use std::thread;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tokio::task;

pub async fn stress_cpu(threads: usize, target_load: f64 ,duration: u64, load_provided: bool, indefinite: bool, stop_flag: Arc<AtomicBool>,task_id: String,) {
    // Error check for target load if load is provided
    if load_provided {
        if target_load < 0.0 || target_load > 100.0 {
            println!("Error: Target load must be between 0 and 100");
            return;
        }

        if target_load == 0.0 {
            println!("Warning: Target load is 0%. The system will not stress the CPU.");
            return;
        }
    }

    if indefinite {
        println!(
            "Running CPU stress test indefinitely. To stop, send a POST request to: http://localhost:8080/stop/{}", task_id);
    }
    // Vector to store thread handles
    let mut handles = Vec::new();

    // Define behavior based on whether load is provided or not
    if load_provided {
        // Time slice logic (if load is provided)
        let load_fraction = target_load / 100.0;

        for thread_id in 0..threads {
            let stop = Arc::clone(&stop_flag);

            let handle = task::spawn_blocking(move || {
                let cycle_time = Duration::from_millis(100);
                let work_time = cycle_time.mul_f64(load_fraction);
                let sleep_time = cycle_time - work_time;

                //global start time
                let start_time = Instant::now();

                while !stop.load(Ordering::SeqCst) {
                    let start = Instant::now();
                    // Work Phase: Simulate CPU-bound work
                    while start.elapsed() < work_time && !stop.load(Ordering::SeqCst) {
                        let _ = (0..1_000_000).fold(0u64, |acc, x| acc.wrapping_add(x));
                    }
                    // Sleep Phase
                    thread::sleep(sleep_time);

                    //if not indefinite, check for time elapsed
                    if !indefinite && start_time.elapsed() >= Duration::from_secs(duration) {
                        break;
                    }
                }

                println!("[Thread {}] Completed busy loop stress.", thread_id);
            });

            handles.push(handle);
        }
    } else {
        // Busy loop with no time slice (if load is not provided)
        for thread_id in 0..threads {
            let stop = Arc::clone(&stop_flag);

            let handle = task::spawn_blocking(move || {
                // If duration is indefinite, don't stop the loop
                if indefinite {
                    while !stop.load(Ordering::SeqCst) {
                        // Simulate CPU-bound work (busy loop)
                        let _ = (0..1_000_000).fold(0u64, |acc, x| acc.wrapping_add(x));
                    }
                } else {
                    // For finite duration, run for the specified time

                    let end_time = Instant::now() + Duration::from_secs(duration);
                    while Instant::now() < end_time && !stop.load(Ordering::SeqCst) {
                        // Simulate CPU-bound work (busy loop)
                        let _ = (0..1_000_000).fold(0u64, |acc, x| acc.wrapping_add(x));
                    }
                }

                println!("[Thread {}] Completed busy loop stress.", thread_id);
            });

            handles.push(handle);
        }
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.await.unwrap();
    }

    println!("CPU stress test completed.");
}

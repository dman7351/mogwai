use std::thread;
use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::env;

/// Simulate high CPU load
fn stress_cpu(threads: usize, duration: u64) {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Stop after specified duration
    let stop_thread = thread::spawn(move || {
        thread::sleep(Duration::from_secs(duration));
        running_clone.store(false, Ordering::SeqCst);
    });

    let mut handles = vec![];

    for _ in 0..threads {
        let running = running.clone();
        handles.push(thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                let _ = (0..1_000_000).fold(0u64, |acc, x| acc.wrapping_add(x));
            }
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }
    let _ = stop_thread.join();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let threads = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(4);
    let duration = args.get(2).and_then(|s| s.parse::<u64>().ok()).unwrap_or(10);

    println!("Starting CPU stress test with {} threads for {} seconds...", threads, duration);
    stress_cpu(threads, duration);
    println!("Stress test completed.");
}

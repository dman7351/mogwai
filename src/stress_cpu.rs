use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

/// Simulate high CPU load
pub fn stress_cpu(threads: usize, duration: u64) {
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
// Read and write speed print statements are commented out because it spams too much
// if you wanna see it, uncomment them


use std::fs::{OpenOptions, remove_file};
use std::io::{Write, Read};
use std::time::{Instant, Duration};
use std::thread::sleep;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::task;

pub async fn stress_disk(
    threads: usize,
    file_size_mb: usize,
    duration: u64,
    stop_flag: Arc<AtomicBool>,
    task_id: String,
) {
    if duration == 0 {
        println!("Running disk stress test indefinitely. To stop, send a POST request to: http://localhost:8080/stop/{}", task_id);
    }

    let mut handles = Vec::new();

    for thread_id in 0..threads {
        let file_name = format!("disk_test_file_{}", thread_id);
        let data = vec![0u8; file_size_mb * 1024 * 1024];
        let stop = Arc::clone(&stop_flag);

        let handle = task::spawn_blocking(move || {
            let start = Instant::now();

            while (duration == 0 || start.elapsed() < Duration::from_secs(duration))
                && !stop.load(Ordering::SeqCst)
            {
                // Write Phase
                if let Ok(mut file) = OpenOptions::new().create(true).write(true).open(&file_name) {
                    let write_start = Instant::now();
                    let _ = file.write_all(&data);
                    let write_time = write_start.elapsed().as_secs_f64();
                    let _write_speed = file_size_mb as f64 / write_time;
                   // println!("[Thread {}] Write speed: {:.2} MB/s", thread_id, write_speed);
                }

                // Read Phase
                let mut buffer = vec![0u8; file_size_mb * 1024 * 1024];
                if let Ok(mut file) = OpenOptions::new().read(true).open(&file_name) {
                    let read_start = Instant::now();
                    let _ = file.read_exact(&mut buffer);
                    let read_time = read_start.elapsed().as_secs_f64();
                    let _read_speed = file_size_mb as f64 / read_time;
                   // println!("[Thread {}] Read speed: {:.2} MB/s", thread_id, read_speed);
                }

                sleep(Duration::from_millis(500));
            }

            println!("[Thread {}] Disk stress test completed.", thread_id);
            if std::path::Path::new(&file_name).exists() {
                let _ = remove_file(&file_name);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Disk stress test finished.");
}

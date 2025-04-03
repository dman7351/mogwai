use std::fs::{OpenOptions, remove_file};
use std::io::{Write, Read};
use std::time::{Instant, Duration};
use std::process;
use std::thread::sleep;
use tokio::task;

pub async fn stress_disk(threads: usize, file_size_mb: usize, duration: u64) {
    if duration == 0 {
        println!("Running disk stress test indefinitely. To stop, use: kill {}", process::id());
    }

    let mut handles = Vec::new();

    for i in 0..threads {
        let file_name = format!("disk_test_file_{}", i);
        let data = vec![0u8; file_size_mb * 1024 * 1024];

        let handle = task::spawn_blocking(move || {
            let start = Instant::now();

            loop {
                // Write Phase
                let mut file = OpenOptions::new().create(true).write(true).open(&file_name).unwrap();
                let write_start = Instant::now();
                file.write_all(&data).unwrap();
                let write_time = write_start.elapsed().as_secs_f64();
                let write_speed = file_size_mb as f64 / write_time;
                println!("[Thread {}] Write speed: {:.2} MB/s", i, write_speed);

                // Read Phase
                let mut buffer = vec![0u8; file_size_mb * 1024 * 1024];
                let mut file = OpenOptions::new().read(true).open(&file_name).unwrap();
                let read_start = Instant::now();
                file.read_exact(&mut buffer).unwrap();
                let read_time = read_start.elapsed().as_secs_f64();
                let read_speed = file_size_mb as f64 / read_time;
                println!("[Thread {}] Read speed: {:.2} MB/s", i, read_speed);

                if duration > 0 && start.elapsed() >= Duration::from_secs(duration) {
                    break;
                }

                sleep(Duration::from_millis(500));
            }

            println!("[Thread {}] Disk stress test completed.", i);
            let _ = remove_file(&file_name);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

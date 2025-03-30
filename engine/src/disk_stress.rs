use std::fs::OpenOptions;
use std::io::{Write, Read};
use std::time::{Instant, Duration};
use std::thread::sleep;
use std::fs;
use std::process;

pub fn stress_disk(file_size_mb: usize, duration: u64) {
    let filename = "disk_test_file";
    let mut file = OpenOptions::new().create(true).write(true).open(filename).unwrap();
    
    println!("Writing {} MB to disk...", file_size_mb);
    
    let data = vec![0u8; file_size_mb * 1024 * 1024];

    // Print PID if running indefinitely
    if duration == 0 {
        println!("Running disk stress test indefinitely. To stop, use: kill {}", process::id());
    }

    let start = Instant::now();
    
    loop {
        let write_start = Instant::now();
        file.write_all(&data).unwrap();
        let write_time = write_start.elapsed().as_secs_f64();

        let write_speed = file_size_mb as f64 / write_time;
        println!("Write speed: {:.2} MB/s", write_speed);

        let mut buffer = vec![0u8; file_size_mb * 1024 * 1024];
        let read_start = Instant::now();
        let mut file = OpenOptions::new().read(true).open(filename).unwrap();
        file.read_exact(&mut buffer).unwrap();
        let read_time = read_start.elapsed().as_secs_f64();

        let read_speed = file_size_mb as f64 / read_time;
        println!("Read speed: {:.2} MB/s", read_speed);

        // If duration is set, stop after reaching it
        if duration > 0 && start.elapsed() >= Duration::from_secs(duration) {
            break;
        }

        sleep(Duration::from_millis(500));
    }

    println!("Disk stress test completed.");
    let _ = fs::remove_file("disk_test_file");
}

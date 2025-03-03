use std::env;
mod cpu_stress;
mod memory_stress;
mod disk_stress;
mod fork_stress;

fn main() {
    let args: Vec<String> = env::args().collect();

    let test_type = &args[1];
    let intensity = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(4);
    let duration = args.get(3).and_then(|s| s.parse::<u64>().ok()).unwrap_or(10);

    match test_type.as_str() {
        "cpu" => {
            println!("Starting CPU stress test with {} threads for {} seconds...", intensity, duration);
            cpu_stress::stress_cpu(intensity, duration);
        }
        "mem" => {
            println!("Starting memory stress test with {} MB for {} seconds...", intensity, duration);
            memory_stress::check_memory_usage();
            memory_stress::stress_memory(intensity, duration);
            memory_stress::check_memory_usage();
        }
        "disk" => {
            println!("Starting disk stress test with {} MB for {} seconds...", intensity, duration);
            disk_stress::stress_disk(intensity, duration);       
        }
        "fork" => {
            println!("Starting fork stress");
            fork_stress::stress_fork();
        }
        
        _ => {
            println!("Invalid test type. Use 'cpu' or 'mem'.");
        }
    }
}

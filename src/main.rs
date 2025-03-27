use clap::Parser;
use std::process;
mod cpu_stress;
mod memory_stress;
mod disk_stress;
mod fork_stress;

/// Struct for handling command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The type of test to run (cpu, mem, disk)
    #[arg(value_name = "TEST_TYPE")]
    test_type: String,

    /// The intensity (number of threads) for the CPU stress test OR max processes for fork stress
    #[arg(short, long, default_value_t = 4)]
    intensity: usize,

    /// Duration of the stress test in seconds (use '0' for indefinite)
    #[arg(short, long, default_value_t = 10)]
    duration: u64,

    /// The load percentage for CPU stress test (1-100%)
    #[arg(short, long)]
    load: Option<f64>,

    /// The size (in MB) of RAM to allocate or size of file to write
    #[arg(short, long, value_name = "MB", default_value_t = 256)]
    size: usize,

    /// Flag to indicate whether to run the fork stress test
    #[arg(short = 'f', long = "fork", help = "Run the fork stress test")]
    fork: bool,

}

fn main() {
    // Define the argument parser using clap
    let args = Args::parse();

    match args.test_type.as_str() {
        "cpu" => {

            if args.fork {
                println!("Running fork stress test... ");
                fork_stress::stress_fork();
            }

            else
            {
            
                let load = args.load.unwrap_or(100.0);
                let indefinite = args.duration == 0;

                println!("Starting CPU stress test with {} threads at {}% load for {} seconds...",
                    args.intensity, load, args.duration
                );

                cpu_stress::stress_cpu(args.intensity, load, args.duration, args.load.is_some(), indefinite);
            }
        }
        "mem" => {
            println!("Starting memory stress test with {} MB for {} seconds...", args.size, args.duration);
            memory_stress::check_memory_usage();
            memory_stress::stress_memory(args.size, args.duration);
            memory_stress::check_memory_usage();
        }
        "disk" => {
            println!("Starting disk stress test with {} MB for {} seconds...", args.size, args.duration);
            disk_stress::stress_disk(args.size, args.duration);       
        }
        
        _ => {
            println!("Invalid test type. Use 'cpu', 'mem', 'disk', or 'fork'.");     
            process::exit(1);   
        }
    }
}

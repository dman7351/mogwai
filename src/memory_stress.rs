//Memory "RAM" stress. The current approach is through the use of a vector, for memory allocation from 'user' input,
//and random generator for accessing memory. Loops through allocated memory to keep active in use, picks a rand index,
//writes to it to prevent OS from optimizing memory allocation.

use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

pub fn stress_memory(mb: usize, duration: u64) {
    println!("Allocating {} MB of memory for {} seconds...", mb, duration);

    let mut sys = System::new_all();
    sys.refresh_memory();
    let available_memory = sys.available_memory() / 1024;
    
    //Error handling.. kills if memory exceeded memmory is reached, prevents from OS crashing.
    if mb as u64 > available_memory{
        eprintln!("Error: Not enought available memory! {} MB, Available {} MB", mb, available_memory);
        return;
    }

    let mut memory_block = Vec::new(); // mutable memory_block, creating a new empty vector
    let chunk_size = mb * 1024 * 1024; // using user MB input for chunk size
    let mut rng = rand::thread_rng(); // random generator for accessing memory

    //println!(
    //    "Memory allocated. Keeping it active for {} seconds...",
    //    duration
    //);
    //let start = Instant::now();

    let start = Instant::now();
    let mut total_allocated = 0;

    //Ram up memory allocation over time given.
    while start.elapsed() < Duration::from_secs(duration) {
        //Graudally allocate memory in chunks based on user input & track total allocation.
        //creating new vector filled with 0's values.
        memory_block.append(&mut vec![0u8; chunk_size]);
        total_allocated += chunk_size / (1024 * 1024);

        //for i in (0..memory_block.len()).step_by(4096) {
        //  memory_block[i] = i as u8;
        //}
        //thread::sleep(Duration::from_millis(500));

        println!(
            "Allocated {} MB. Total allocated: {} MB.",
            chunk_size / (1024 * 1024),
            total_allocated
        );

        //actively use the memory by randomly accessing a portion of it.
        //access a larger portion of the memory, and at a rand index we modify the value by 1kb to keep
        //active memory usage.
        for _ in 0..(memory_block.len() / 1024) {
            let i = rng.gen_range(0..memory_block.len());
            memory_block[i] = i as u8;
        }

        check_memory_usage();
        thread::sleep(Duration::from_secs(1)); //slow down memory allocation to prevent instant usage.
    }

    println!("Memory stress test completed.");
}

pub fn check_memory_usage() {
    let mut sys = System::new_all();
    sys.refresh_memory();

    println!(
        "Total Memory: {} MB |  Used Memory: {} MB",
        sys.total_memory() / 1024,
        sys.used_memory() / 1024
    );

}

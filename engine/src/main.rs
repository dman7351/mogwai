use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;
use serde::Deserialize;
use std::sync::{Arc, atomic::AtomicBool};

mod thread_manager;
use thread_manager::{ GLOBAL_REGISTRY};
mod cpu_stress;
mod memory_stress;
mod disk_stress;
mod fork_stress;

#[derive(Deserialize)]
struct TestParams {
    intensity: Option<usize>,
    duration: Option<u64>,
    load: Option<f64>,
    size: Option<usize>,
    fork: Option<bool>,
}

async fn start_cpu_stress_test(
    params: web::Json<TestParams>,
) -> impl Responder {
    let intensity = params.intensity.unwrap_or(4);
    let duration = params.duration.unwrap_or(10);
    let load = params.load.unwrap_or(100.0);
    let indefinite = duration == 0;
    let task_id = thread_manager::generate_task_id("cpu");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = stop_flag.clone();
    

    let handle = {
        let task_id = task_id.clone(); // clone scoped for async block

        tokio::spawn(async move {
            // Check if the fork flag is set in the request
            if let Some(fork) = params.fork {
                if fork {
                    // Trigger fork stress logic
                    println!(
                        "Starting fork stress test with {} processes for {} seconds...",
                        intensity, duration
                    );
                    fork_stress::stress_fork(intensity, duration);
                } else {
                    // Trigger regular CPU stress logic if fork is false
                    println!(
                        "Starting CPU stress test with {} threads at {}% load for {} seconds...",
                        intensity, load, duration
                    );
                    cpu_stress::stress_cpu(intensity, load, duration, params.load.is_some(), indefinite, flag_clone, task_id.clone()).await;
                }
            } else {
                // No fork flag was provided, so run the regular CPU stress test
                println!(
                    "No fork flag provided. Starting regular CPU stress test with {} threads at {}% load for {} seconds...",
                    intensity, load, duration
                );
                cpu_stress::stress_cpu(intensity, load, duration, params.load.is_some(), indefinite, flag_clone, task_id.clone()).await;
            }

            println!("[{}] CPU stress test finished", task_id);
        })
    };

    thread_manager::register_task(task_id.clone(), handle, stop_flag);
    

    HttpResponse::Ok().body(format!("CPU stress task started with ID: {}", task_id))
}

async fn start_memory_stress_test(
    params: web::Json<TestParams>,
) -> impl Responder {
    let intensity = params.intensity.unwrap_or(4);
    let duration = params.duration.unwrap_or(10);
    let size = params.size.unwrap_or(256);
    let task_id = thread_manager::generate_task_id("mem"); 

    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = stop_flag.clone();

    let handle = {
        let task_id = task_id.clone(); // clone scoped for async block

        tokio::spawn(async move {
            println!(
                "Starting memory stress test with {} MB for {} seconds...",
                size, duration
            );
            memory_stress::check_memory_usage();
            memory_stress::stress_memory(intensity, size, duration, flag_clone, task_id.clone()).await;
            memory_stress::check_memory_usage();
            println!("- Memory stress test ID: \"{}\" finished", task_id);
        })
    };

    thread_manager::register_task(task_id.clone(), handle, stop_flag);


    HttpResponse::Ok().body(format!("Memory stress task started with ID: {}", task_id))
}

async fn start_disk_stress_test(
    params: web::Json<TestParams>,
) -> impl Responder {
    let intensity = params.intensity.unwrap_or(4);
    let duration = params.duration.unwrap_or(10);
    let size = params.size.unwrap_or(256);
    let task_id = thread_manager::generate_task_id("disk");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = stop_flag.clone();

    let handle = {
        let task_id = task_id.clone(); // clone scoped for async block

        tokio::spawn(async move {
            println!(
                "Starting disk stress test with {} MB for {} seconds...",
                size, duration
            );
            disk_stress::stress_disk(intensity, size, duration, flag_clone, task_id.clone()).await;
            println!("[{}] Disk stress test finished", task_id);
        })
    };

    thread_manager::register_task(task_id.clone(), handle, stop_flag);


    HttpResponse::Ok().body(format!("Disk stress task started with ID: {}", task_id))
}

// Task listing
async fn list_running_tasks() -> impl Responder {
    let registry = &GLOBAL_REGISTRY;
    let lock = registry.lock().unwrap();
    println!("-> GET/tasks: {:?}", lock.keys());
    drop(lock);
    HttpResponse::Ok().json(thread_manager::list_tasks(registry))
}

// Task stopping
async fn stop_running_task(id: web::Path<String>) -> impl Responder {
    thread_manager::stop_task(&id, &GLOBAL_REGISTRY);
    HttpResponse::Ok().body(format!("-> POST/stop{} request sent", id))
}

async fn stop_all_tasks() -> impl Responder {
    use thread_manager::GLOBAL_REGISTRY;
    let registry = &GLOBAL_REGISTRY;
    let task_ids = thread_manager::list_tasks(registry);

    for id in &task_ids {
        thread_manager::stop_task(id, registry);
    }

    HttpResponse::Ok().body(format!("-> POST/stop-all request sent to all {} tasks", task_ids.len()))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Setup HTTP server to handle requests
    HttpServer::new(move || {
        //using move to transfer ownership of task registry
        App::new()
            .wrap(Cors::default()
                .allow_any_origin()  // Allows any origin (for development)
                .allow_any_method()  // Allows any HTTP method (GET, POST, etc.)
                .allow_any_header()  // Allows any headers
                .max_age(3600))
            .route("/cpu-stress", web::post().to(start_cpu_stress_test))
            .route("/mem-stress", web::post().to(start_memory_stress_test))
            .route("/disk-stress", web::post().to(start_disk_stress_test))
            .route("/tasks", web::get().to(list_running_tasks))
            .route("/stop/{id}", web::post().to(stop_running_task))
            .route("/stop-all", web::post().to(stop_all_tasks))
    })
    .bind("0.0.0.0:8080")?  // Expose on port 8080
    .run()
    .await
}

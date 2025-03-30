use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;
use serde::Deserialize;
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

async fn start_cpu_stress_test(params: web::Json<TestParams>) -> impl Responder {
    let intensity = params.intensity.unwrap_or(4); // default CPU threads
    let duration = params.duration.unwrap_or(10);  // default duration
    let load = params.load.unwrap_or(100.0);
    let indefinite = duration == 0;

    // Check if the fork flag is set in the request
    if let Some(fork) = params.fork {
        if fork {
            // Trigger fork stress logic
            println!("Starting fork stress test with {} processes for {} seconds...", intensity, duration);
            fork_stress::stress_fork(intensity, duration);
        } else {
            // Trigger regular CPU stress logic if fork is false
            println!("Starting CPU stress test with {} threads at {}% load for {} seconds...", intensity, load, duration);
            cpu_stress::stress_cpu(intensity, load, duration, params.load.is_some(), indefinite).await;
        }
    } else {
        // No fork flag was provided, so run the regular CPU stress test
        println!("No fork flag provided. Starting regular CPU stress test with {} threads at {}% load for {} seconds...", intensity, load, duration);
        cpu_stress::stress_cpu(intensity, load, duration, params.load.is_some(), indefinite).await;
    }

    HttpResponse::Ok().body("CPU stress test finished\n")
}

async fn start_memory_stress_test(params: web::Json<TestParams>) -> impl Responder {
    let duration = params.duration.unwrap_or(10);
    let size = params.size.unwrap_or(256);
    println!("Starting memory stress test with {} MB for {} seconds...", size, duration);
    memory_stress::check_memory_usage();
    memory_stress::stress_memory(size, duration);
    memory_stress::check_memory_usage();

    HttpResponse::Ok().body("Memory stress test finished\n")
}

async fn start_disk_stress_test(params: web::Json<TestParams>) -> impl Responder {
    let duration = params.duration.unwrap_or(10);
    let size = params.size.unwrap_or(256);
    println!("Starting disk stress test with {} MB for {} seconds...", size, duration);
    disk_stress::stress_disk(size, duration);

    HttpResponse::Ok().body("Disk stress test finished\n")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Setup HTTP server to handle requests
    HttpServer::new(|| {
        App::new()
             .wrap(Cors::default()
                .allow_any_origin()  // Allows any origin (for development)
                .allow_any_method()  // Allows any HTTP method (GET, POST, etc.)
                .allow_any_header()  // Allows any headers
                .max_age(3600))
            .route("/cpu-stress", web::post().to(start_cpu_stress_test))
            .route("/mem-stress", web::post().to(start_memory_stress_test))
            .route("/disk-stress", web::post().to(start_disk_stress_test))
    })
    .bind("0.0.0.0:8080")?  // Expose on port 8080
    .run()
    .await
}

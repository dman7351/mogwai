use actix_cors::Cors;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Deserialize, Serialize)]
struct TestParams {
    intensity: Option<u32>,        // Default: 4
    duration: Option<u32>,         // Default: 10
    load: Option<f32>,             // Default: 100.0
    size: Option<u32>,             // Default: 256
    fork: Option<bool>,            // Default: false
}

impl Default for TestParams {
    fn default() -> Self {
        Self {
            intensity: Some(4),
            duration: Some(10),
            load: Some(100.0),
            size: Some(256),
            fork: Some(false),
        }
    }
}

#[post("/cpu-stress")]
async fn cpu_stress(params: web::Json<TestParams>, client: web::Data<Client>) -> impl Responder {
    println!(
        "Starting CPU stress test with intensity: {:?}, duration: {:?}, load: {:?}",
        params.intensity, params.duration, params.load
    );

    let url = "http://engine-service:8080/cpu-stress";
    let res = client
        .post(url)
        .json(&*params)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                HttpResponse::Ok().body("CPU stress test started.")
            } else {
                HttpResponse::InternalServerError().body("Failed to start CPU stress test")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Request error"),
    }
}

#[post("/mem-stress")]
async fn mem_stress(params: web::Json<TestParams>, client: web::Data<Client>) -> impl Responder {
    println!(
        "Starting memory stress test with size: {:?} MB, duration: {:?} seconds.",
        params.size, params.duration
    );

    let url = "http://engine-service:8080/mem-stress";
    let res = client
        .post(url)
        .json(&*params)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                HttpResponse::Ok().body("Memory stress test started.")
            } else {
                HttpResponse::InternalServerError().body("Failed to start memory stress test")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Request error"),
    }
}

#[post("/disk-stress")]
async fn disk_stress(params: web::Json<TestParams>, client: web::Data<Client>) -> impl Responder {
    println!(
        "Starting disk stress test with size: {:?} MB, duration: {:?} seconds.",
        params.size, params.duration
    );

    let url = "http://engine-service:8080/disk-stress";
    let res = client
        .post(url)
        .json(&*params)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                HttpResponse::Ok().body("Disk stress test started.")
            } else {
                HttpResponse::InternalServerError().body("Failed to start disk stress test")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Request error"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::new();

    HttpServer::new(move || {
        let cors = Cors::permissive(); // Same as `allow_origins = ["*"]` in FastAPI

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(client.clone()))
            .service(cpu_stress)
            .service(mem_stress)
            .service(disk_stress)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}

use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;

use std::collections::BTreeMap;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{Client as KubeClient, api::{Api, PostParams, ObjectMeta}};
use k8s_openapi::api::core::v1::{Node, Pod, PodSpec, Container, LocalObjectReference, Service, ServiceSpec, ServicePort};


#[derive(Debug, Deserialize, Serialize)]
struct TestParams {
    intensity: Option<u32>,        // Default: 4
    duration: Option<u32>,         // Default: 10
    load: Option<f32>,             // Default: 100.0
    size: Option<u32>,             // Default: 256
    fork: Option<bool>,            // Default: false
    node: String
}

impl Default for TestParams {
    fn default() -> Self {
        Self {
            intensity: Some(4),
            duration: Some(10),
            load: Some(100.0),
            size: Some(256),
            fork: Some(false),
            node: "UNSET".to_string(),
        }
    }
}

#[derive(Serialize)]
struct NodeInfo {
    name:String
}

#[derive(Debug, Deserialize)]
struct SpawnRequest {
    node_name: String,
}

#[get("/nodes")]
async fn list_nodes() -> impl Responder {
    let client = match KubeClient::try_default().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to create client: {}", e)),
    };

    let nodes: Api<Node> = Api::all(client);

    match nodes.list(&Default::default()).await {
        Ok(node_list) => {
            let node_names: Vec<NodeInfo> = node_list.items.into_iter().filter_map(|n| {
                n.metadata.name.clone().map(|name| NodeInfo { name })
            }).collect();

            HttpResponse::Ok().json(node_names)
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to list nodes: {}", e)),
    }
}

#[post("/spawn-engine")]
async fn spawn_engine(
    payload: web::Json<SpawnRequest>,
) -> impl Responder {
    let client = match KubeClient::try_default().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Client error: {}", e)),
    };

    let pod_name = format!("mogwai-engine-{}", payload.node_name);
    let label_key = "stateful-id";

    let pods: Api<Pod> = Api::namespaced(client.clone(), "default");

    let pod = Pod {
        metadata: ObjectMeta {
            name: Some(pod_name.clone()),
            labels: Some(BTreeMap::from([
                ("app".to_string(), "mogwai-engine".to_string()),
                (label_key.to_string(), pod_name.clone()),
            ])),
            ..Default::default()
        },
        spec: Some(PodSpec {
            containers: vec![Container {
                name: "engine-container".to_string(),
                image: Some("ghcr.io/dman7351/stress-test:dev".to_string()),
                image_pull_policy: Some("Always".to_string()),
                ports: Some(vec![k8s_openapi::api::core::v1::ContainerPort {
                    container_port: 8080,
                    ..Default::default()
                }]),
                ..Default::default()
            }],
            node_name: Some(payload.node_name.clone()),
            restart_policy: Some("Never".into()),
            image_pull_secrets: Some(vec![LocalObjectReference {
                name: "github-registry-secret".to_string(),
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

     if let Err(e) = pods.create(&PostParams::default(), &pod).await {
        return HttpResponse::InternalServerError().body(format!("Pod creation failed: {}", e));
    }

    let services: Api<Service> = Api::namespaced(client.clone(), "default");
    let svc = Service {
        metadata: ObjectMeta {
            name: Some(pod_name.clone()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            selector: Some(BTreeMap::from([
                (label_key.to_string(), pod_name.clone()),
            ])),
            cluster_ip: Some("None".to_string()), // headless
            ports: Some(vec![ServicePort {
                port: 8080,
                target_port: Some(IntOrString::Int(8080)),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    match services.create(&PostParams::default(), &svc).await {
        Ok(_) => HttpResponse::Ok().body("Engine pod and headless service spawned."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service creation failed: {}", e)),
    }
}

#[post("/cpu-stress")]
async fn cpu_stress(params: web::Json<TestParams>, client: web::Data<HttpClient>) -> impl Responder {
    println!(
        "Starting CPU stress test on node {} with intensity: {:?}, duration: {:?}, load: {:?}",
        params.node, params.intensity, params.duration, params.load
    );

    let url = format!("http://mogwai-engine-{}.default.svc.cluster.local:8080/cpu-stress", params.node);

    match client.post(&url).json(&*params).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            HttpResponse::build(status).body(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Request failed: {}", e)),
    }
}

#[post("/mem-stress")]
async fn mem_stress(params: web::Json<TestParams>, client: web::Data<HttpClient>) -> impl Responder {
    println!(
        "Starting memory stress test on node {} with intensity: {:?}, duration: {:?}, size: {:?}",
        params.node, params.intensity, params.duration, params.size
    );

    let url = format!("http://mogwai-engine-{}.default.svc.cluster.local:8080/mem-stress", params.node);

    match client.post(&url).json(&*params).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            HttpResponse::build(status).body(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Request failed: {}", e)),
    }
}

#[post("/disk-stress")]
async fn disk_stress(params: web::Json<TestParams>, client: web::Data<HttpClient>) -> impl Responder {
    println!(
        "Starting disk stress test on node {} with intensity: {:?}, duration: {:?}, size: {:?}",
        params.node, params.intensity, params.duration, params.size
    );

    let url = format!("http://mogwai-engine-{}.default.svc.cluster.local:8080/disk-stress", params.node);

    match client.post(&url).json(&*params).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            HttpResponse::build(status).body(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Request failed: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = HttpClient::new();
    println!("Starting controller server on 0.0.0.0:8081");
    HttpServer::new(move || {
        let cors = Cors::permissive(); // Same as `allow_origins = ["*"]` in FastAPI

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(client.clone()))
            .service(cpu_stress)
            .service(mem_stress)
            .service(disk_stress)
            .service(list_nodes)
            .service(spawn_engine)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}

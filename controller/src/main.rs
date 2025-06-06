// Import necessary crates
use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;

use std::collections::BTreeMap;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{Client as KubeClient, api::{Api, PostParams, ObjectMeta, ListParams, DeleteParams}};
use k8s_openapi::api::core::v1::{Node, Pod, PodSpec, Container, LocalObjectReference, Service, ServiceSpec, ServicePort};
use futures::future::join_all;

// Struct used to receive and pass stress test parameters
#[derive(Debug, Deserialize, Serialize)]
struct TestParams {
    intensity: Option<u32>, // Number of threads or operations, default: 4
    duration: Option<u32>,  // Duration of the test in seconds, default: 10
    load: Option<f32>,      // Load percentage for CPU stress, default: 100.0
    size: Option<u32>,      // Size in MB (for memory/disk stress), default: 256
    fork: Option<bool>,     // Whether to fork processes (for fork stress), default: false
    node: String            // Target node name for the test
}

// Provide default values for TestParams fields
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

// Struct to serialize node info in response
#[derive(Serialize)]
struct NodeInfo {
    name: String
}

// Struct used for requests that include a node name
#[derive(Debug, Deserialize)]
struct NodeRequest {
    node_name: String,
}

// GET /nodes — List all node names in the Kubernetes cluster
#[get("/nodes")]
async fn list_nodes() -> impl Responder {
    let client = match KubeClient::try_default().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to create client: {}", e)),
    };

    let nodes: Api<Node> = Api::all(client);

    match nodes.list(&Default::default()).await {
        Ok(node_list) => {
            // Extract node names into a Vec
            let node_names: Vec<NodeInfo> = node_list.items.into_iter().filter_map(|n| {
                n.metadata.name.clone().map(|name| NodeInfo { name })
            }).collect();

            HttpResponse::Ok().json(node_names)
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to list nodes: {}", e)),
    }
}

// POST /spawn-engine — Spawn a pod and a headless service on a specific node
#[post("/spawn-engine")]
async fn spawn_engine(
    payload: web::Json<NodeRequest>,
) -> impl Responder {
    // Initialize Kubernetes client
    let client = match KubeClient::try_default().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Client error: {}", e)),
    };

    // Generate pod name from node
    let pod_name = format!("mogwai-engine-{}", payload.node_name);
    let label_key = "stateful-id";

    let pods: Api<Pod> = Api::namespaced(client.clone(), "default");

    // Define pod specification
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
                image: Some("ghcr.io/dman7351/mogwai-engine:latest".to_string()),
                image_pull_policy: Some("Always".to_string()),
                ports: Some(vec![k8s_openapi::api::core::v1::ContainerPort {
                    container_port: 8080,
                    ..Default::default()
                }]),
                ..Default::default()
            }],
            node_name: Some(payload.node_name.clone()), // Assign pod to the requested node
            restart_policy: Some("Never".into()),
            image_pull_secrets: Some(vec![LocalObjectReference {
                name: "github-registry-secret".to_string(),
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create the pod in Kubernetes
    if let Err(e) = pods.create(&PostParams::default(), &pod).await {
        return HttpResponse::InternalServerError().body(format!("Pod creation failed: {}", e));
    }

    // Define and create a headless service for direct DNS-based access
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
            cluster_ip: Some("None".to_string()), // Headless service
            ports: Some(vec![ServicePort {
                port: 8080,
                target_port: Some(IntOrString::Int(8080)),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create the service
    match services.create(&PostParams::default(), &svc).await {
        Ok(_) => HttpResponse::Ok().body("Engine pod and headless service spawned."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Service creation failed: {}", e)),
    }
}

// POST /remove-engine — Delete the pod and service for a given node
#[post("/remove-engine")]
async fn remove_engine(
    payload: web::Json<NodeRequest>,
) -> impl Responder {
    let client = match KubeClient::try_default().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Client error: {}", e)),
    };

    let pod_name = format!("mogwai-engine-{}", payload.node_name);

    let pods: Api<Pod> = Api::namespaced(client.clone(), "default");
    let services: Api<Service> = Api::namespaced(client.clone(), "default");

    // Attempt to delete the pod and service
    let pod_result = pods.delete(&pod_name, &DeleteParams::default()).await;
    let svc_result = services.delete(&pod_name, &DeleteParams::default()).await;

    // Prepare response messages
    let pod_msg = match pod_result {
        Ok(_) => format!("Pod {} deletion initiated.", pod_name),
        Err(e) => format!("Pod deletion error: {}", e),
    };
    let svc_msg = match svc_result {
        Ok(_) => format!("Service {} deletion initiated.", pod_name),
        Err(e) => format!("Service deletion error: {}", e),
    };

    HttpResponse::Ok().json(serde_json::json!({
        "pod": pod_msg,
        "service": svc_msg
    }))
}

// POST /cpu-stress — Send a stress request to the engine pod on a specific node
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

// POST /mem-stress — Trigger memory stress test
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

// POST /disk-stress — Trigger disk I/O stress test
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

// POST /tasks/{node} — Get list of running tasks from engine pod on a node
#[post("/tasks/{node}")]
async fn list_tasks(path: web::Path<String>, client: web::Data<HttpClient>) -> impl Responder {
    let node = path.into_inner();
    let url = format!("http://mogwai-engine-{}.default.svc.cluster.local:8080/tasks", node);

    match client.get(&url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            HttpResponse::build(status).body(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Request failed: {}", e)),
    }
}

// POST /stop/{node}/{id} — Stop a specific task by ID on a node
#[post("/stop/{node}/{id}")]
async fn stop_task(path: web::Path<(String, String)>, client: web::Data<HttpClient>) -> impl Responder {
    let (node, id) = path.into_inner();
    let url = format!("http://mogwai-engine-{}.default.svc.cluster.local:8080/stop/{}", node, id);

    match client.post(&url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            HttpResponse::build(status).body(body)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Request failed: {}", e)),
    }
}

// POST /stop-all — Send stop-all command to every running engine pod
#[post("/stop-all")]
async fn stop_all_tasks(client: web::Data<HttpClient>) -> impl Responder {
    let kube_client = match KubeClient::try_default().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to create Kube client: {}", e)),
    };

    let pods_api: Api<Pod> = Api::namespaced(kube_client.clone(), "default");
    let lp = ListParams::default().labels("app=mogwai-engine");

    // List all mogwai-engine pods
    let pods = match pods_api.list(&lp).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to list mogwai-engine pods: {}", e)),
    };

    // Extract node names from pods
    let target_nodes: Vec<String> = pods.items.into_iter()
        .filter_map(|pod| pod.spec.and_then(|spec| spec.node_name))
        .collect();

    if target_nodes.is_empty() {
        return HttpResponse::Ok().body("No mogwai-engine pods found on any nodes.");
    }

    // Send stop-all to each node in parallel
    let tasks = target_nodes.iter().map(|node| {
        let url = format!("http://mogwai-engine-{}.default.svc.cluster.local:8080/stop-all", node);
        let client = client.clone();
        let node = node.clone();

        async move {
            match client.post(&url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    format!("{}: {} - {}", node, status, body)
                }
                Err(e) => format!("{}: FAILED - {}", node, e),
            }
        }
    });
    let results: Vec<String> = join_all(tasks).await;
    HttpResponse::Ok().json(results)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = HttpClient::new();
    println!("Starting controller server on 0.0.0.0:8081");
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(client.clone()))
            .service(cpu_stress)
            .service(mem_stress)
            .service(disk_stress)
            .service(list_nodes)
            .service(spawn_engine)
            .service(remove_engine)
            .service(list_tasks)
            .service(stop_task)
            .service(stop_all_tasks)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
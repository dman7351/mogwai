use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::task::JoinHandle;
use once_cell::sync::Lazy;

static TASK_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub static GLOBAL_REGISTRY: Lazy<TaskRegistry> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

pub type TaskRegistry = Arc<Mutex<HashMap<String, (JoinHandle<()>, Arc<AtomicBool>)>>>;


pub fn generate_task_id(prefix: &str) -> String {
    let id = TASK_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}-{}", prefix, id)
}

pub fn register_task(
    id: String,
    handle: JoinHandle<()>,
    stop_flag: Arc<AtomicBool>,
) {
    let registry = &GLOBAL_REGISTRY;

    // dummy placeholder
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    {
        let mut guard = registry.lock().unwrap();
        guard.insert(id.clone(), (tokio::spawn(async { let _ = rx.await; }), stop_flag.clone()));
        println!("- Task registered: {} | Total now: {}", id, guard.len());
    }

    let registry_clone = Arc::clone(registry);
    let id_clone = id.clone();

    tokio::spawn(async move {
        let _ = handle.await;

        let mut guard = registry_clone.lock().unwrap();
        guard.remove(&id_clone);
        println!("- Cleaned up finished task: {}", id_clone);
    });

    // lets the dummy task exit immediately
    drop(tx);
}





pub fn stop_task(id: &str, registry: &TaskRegistry) {
    if let Some((_, flag)) = registry.lock().unwrap().get(id) {
        flag.store(true, Ordering::SeqCst);
    }
}

pub fn list_tasks(registry: &TaskRegistry) -> Vec<String> {
    let guard = registry.lock().unwrap();
    let keys: Vec<String> = guard.keys().cloned().collect();
    keys
}

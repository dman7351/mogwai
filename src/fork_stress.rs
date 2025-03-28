use std::process::exit;
use std::thread;
use std::time::Duration;
use libc::{fork, waitpid, c_int};

pub fn stress_fork(num_processes: usize, duration: u64) {
    let mut children = vec![];

    for _ in 0..num_processes {
        unsafe {
            let pid = fork();
            if pid == 0 {
                // Child process
                thread::sleep(Duration::from_secs(duration));
                exit(0);
            } else if pid > 0 {
                // Parent process
                children.push(pid);
            } else {
                eprintln!("Fork failed");
                exit(1);
            }
        }
    }

    // Parent waits for all children
    for pid in children {
        unsafe {
            let mut status: c_int = 0;
            waitpid(pid, &mut status, 0);
        }
    }
}

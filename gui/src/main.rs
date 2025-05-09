mod gui;

fn main() {
    match gui::run() {
        Ok(_) => println!("Application exited successfully"),
        Err(e) => println!("Error running application: {}", e),
    }
}
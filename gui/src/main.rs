/**
 * Mogwai Test GUI - Performance Testing Utility
 * CS 488 Senior Project/Final 
 * main.rs
 * This is the main.rs for our gui application, which provides a graphical interface for running CPU, memory,
 * and disk stress tests across different environments.
 */
 // Import the GUI module.
mod gui;

/**
 * Main function - application entry point
 * This function:
 * 1. Calls the run() function from the gui module to start the application
 * 2. Handles any errors that might occur during execution
 * 3. Provides user feedback on application exit status
 */
fn main() {
    // Call the gui::run() function and handle the result.
    match gui::run() {
        // If application exits normally, display success message.
        Ok(_) => println!("Application exited successfully"),
        // If an error occurs, display it to the console
        Err(e) => println!("Error running application: {}", e),
    }
}

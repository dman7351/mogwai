//to run cargo run --bin cli
// Import necessary libraries:
// - std::io - For handling input/output operations
// - std::sync - For thread synchronization primitives (Arc = atomic reference counting, Mutex = mutual exclusion)
// - tokio::runtime - For asynchronous task execution
// - std::thread - For spawning and managing threads
// - std::time - For time-related operations and tracking
// - chrono - For more advanced date/time handling and formatting
// - reqwest - HTTP client for making API requests
// - serde - For serializing/deserializing data structures
// - uuid - For generating unique identifiers
// - std::process - For executing external commands
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{Local, NaiveTime, TimeZone};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::process::Command;

// TestParams structure - Defines the parameters for a stress test
// This structure stores all possible configuration options for any type of test
// The #[derive] attributes enable automatic serialization for sending over HTTP
#[derive(Debug, Clone, Serialize)]
struct TestParams {
    id: String,          // Unique identifier for the test
    name: String,        // Human-readable name for the test
    test_type: String,   // Type of test (cpu, mem, disk)
    threads: Option<u32>, // Number of threads to use (Optional)
    duration: u32,       // Duration of the test in seconds
    load: Option<u32>,   // CPU load percentage (Optional - used for CPU tests)
    size: Option<u32>,   // Size in MB (Optional - used for memory and disk tests)
    fork: Option<bool>,  // Whether to fork processes (Optional - used for CPU tests)
    scheduled_time: Option<u64>, // Unix timestamp for scheduled execution (Optional)
    node: String,        // Target node where the test will run
}

// TestRequest structure - Simplified version of TestParams for API requests
// This structure maps our internal parameters to the format expected by the API
#[derive(Serialize)]
struct TestRequest {
    id: String,          // Unique identifier for the test
    name: String,        // Human-readable name for the test
    intensity: Option<u32>, // Renamed from 'threads' to match API expectations
    duration: u32,       // Duration of the test in seconds
    load: Option<u32>,   // CPU load percentage (Optional)
    size: Option<u32>,   // Size in MB (Optional)
    fork: Option<bool>,  // Whether to fork processes (Optional)
    node: String,        // Target node
}

// AiResponse structure - Format of responses from the AI test generator
// Used to deserialize the JSON responses from mogAI.py
#[derive(Deserialize)]
struct AiResponse {
    test_type: String,   // Type of test (cpu, mem, disk)
    #[serde(default)]    // Default to 0 if not provided
    threads: u32,        // Number of threads to use
    duration: u32,       // Duration of the test in seconds
    #[serde(default)]    // Default to None if not provided
    load: Option<u32>,   // CPU load percentage (Optional)
    #[serde(default)]    // Default to None if not provided
    size: Option<u32>,   // Size in MB (Optional)
    #[serde(default)]    // Default to None if not provided
    fork: Option<bool>,  // Whether to fork processes (Optional)
    #[serde(default)]    // Default to 0 if not provided
    intensity: u32,      // Intensity level from AI recommendation - ignored on purpose
}

// Main function - Entry point of the application
fn main() {
    // Display an ASCII art logo and welcome message
    // This provides a visual identity to the CLI tool
    println!(
        "
              #              
            +  :+            
            : : :            
           # -   *           
          %::     #          
          *       :          
          :-      :@         
         =  :   :: :         
        +   : .:    =        
        -    :.     :        
       %      :.     *       
      @:        :     #      
      =       :: .:   :      
      :    ::      :: .@     
     =   -.           :=     
    * .-                =    
    =:  .::.......::.   :    
   %.::               :: #   
   =:                   ::@  
  =                       =  
 *                         = 
*.                          +
- .#%#=:             :=*%%. -
= :@@@@@@*.        *@@@@@@- -
=  @@@@@@@@=     =@@@@@@@@. =
%:  %@@@@@@@*   =@@@@@@@%. :%
  :  -%@@@@@@   @@@@@@%-  :  
  @-     :::     :::     =   
    +                   =    
      -               :      
       *:           :*       
         =.       .=         
           @*-::+@
         ==============================================\n\
         === Welcome to the System Stress Test CLI ===\n\
         ==============================================\n\
         This tool will help you run various stress tests on your system.\n\
         You can schedule tests, view them, or change server settings.\n"
    );

    // Prompt user for server URL with a default of http://localhost:8080
    let mut server_url = get_server_url();
    println!("\nUsing server at: {}\n", server_url);

    // Set a default node for tests to run on (in this case, minikube) - unused mut on purpose
    let mut default_node = "minikube";

    // Create a shared collection for scheduled tests
    // Arc provides thread-safe reference counting, allowing multiple threads to safely access the data
    // Mutex ensures only one thread can modify the data at a time
    let scheduled_tests = Arc::new(Mutex::new(Vec::<TestParams>::new()));

    // Start a background thread to monitor and execute scheduled tests
    // This thread runs continuously and checks if any tests are due to run
    let tests_to_run = Arc::clone(&scheduled_tests);
    let server_url_clone = server_url.clone();
    let _execution_thread = thread::spawn(move || {
        // Create a Tokio runtime for handling async operations within this thread
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Create an HTTP client with a timeout for API requests
            let client = Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap();

            // Continuous loop to check for and execute scheduled tests
            loop {
                // Get current time as Unix timestamp
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let mut tests_to_execute = Vec::new();

                // Check for tests that are ready to run:
                // - Tests with no scheduled_time should run immediately
                // - Tests with scheduled_time should run if current_time has reached that time
                {
                    // Lock the shared collection to safely modify it
                    let mut tests = tests_to_run.lock().unwrap();
                    let mut i = 0;
                    while i < tests.len() {
                        if let Some(scheduled_time) = tests[i].scheduled_time {
                            if current_time >= scheduled_time {
                                // Move the test from the scheduled list to the execution list
                                tests_to_execute.push(tests.remove(i));
                            } else {
                                i += 1;
                            }
                        } else {
                            // Test with no scheduled time - run immediately
                            tests_to_execute.push(tests.remove(i));
                        }
                    }
                }

                // Execute tests concurrently using Tokio tasks
                let mut handles = Vec::new();
                for test in tests_to_execute {
                    // Clone resources needed for the task
                    let client_clone = client.clone();
                    let url_clone = server_url_clone.clone();
                    let test_clone = test.clone();
                    
                    // Spawn an async task for each test
                    let handle = tokio::spawn(async move {
                        // Run the test and wait for it to complete
                        run_test(&client_clone, &url_clone, &test_clone).await;
                        println!("\nTest completed. Returning to main menu...");
                        
                        // Display the menu again after test completion
                        println!("\n----------------------------------------------");
                        println!("Main Menu:");
                        println!("1. Schedule a new test");
                        println!("2. View scheduled tests");
                        println!("3. Change server URL (current: {})", url_clone);
                        println!("4. Change default node (default: minikube)");
                        println!("5. Run AI test");
                        println!("6. Exit");
                        print!("Enter your choice (1-6): ");
                        io::stdout().flush().unwrap();
        
                    });
                    handles.push(handle);
                }

                // Wait for all test execution tasks to complete
                for handle in handles {
                    let _ = handle.await;
                }

                // Brief pause before the next check to prevent CPU overuse
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    });

    // Main menu loop - This is the primary user interface
    // The loop continues until the user chooses to exit
    loop {
        // Display menu options
        println!("\n----------------------------------------------");
        println!("Main Menu:");
        println!("1. Schedule a new test");
        println!("2. View scheduled tests");
        println!("3. Change server URL (current: {})", server_url);
        println!("4. Change default node (default: {})", default_node);
        println!("5. Run AI test");
        println!("6. Exit");
        print!("Enter your choice (1-6): ");
        io::stdout().flush().unwrap();

        // Read user input
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        // Process the user's selection
        match choice.trim() {
            "1" => {
                // Schedule a new test by collecting parameters and adding to the scheduled list
                if let Some(test_params) = collect_test_params(default_node) {
                    scheduled_tests.lock().unwrap().push(test_params);
                }
            }
            "2" => {
                // View all currently scheduled tests
                let tests = scheduled_tests.lock().unwrap();
                if tests.is_empty() {
                    println!("\nNo tests currently scheduled.");
                } else {
                    println!("\n=== Scheduled Tests ===");
                    for (i, test) in tests.iter().enumerate() {
                        // Display scheduled time if present, otherwise show "Run immediately"
                        if let Some(time) = test.scheduled_time {
                            // Convert Unix timestamp to human-readable format
                            let dt = Local.timestamp_opt(time as i64, 0).unwrap();
                            println!(
                                "\n{}. [{}] {} Test - Duration: {}s - Scheduled for: {}",
                                i + 1,
                                test.id,
                                test.test_type.to_uppercase(),
                                test.duration,
                                dt.format("%Y-%m-%d %H:%M:%S")
                            );
                        } else {
                            println!(
                                "\n{}. [{}] {} Test - Duration: {}s - Run immediately",
                                i + 1,
                                test.id,
                                test.test_type.to_uppercase(),
                                test.duration
                            );
                        }
                    }
                }
                
                // Pause for user to review the list before returning to menu
                println!("\nPress Enter to return to the main menu...");
                let mut _pause = String::new();
                io::stdin().read_line(&mut _pause).unwrap();
            },
            "3" => {
                // Change the server URL
                server_url = get_server_url();
                println!("\nServer URL changed to: {}", server_url);
            }
            "4" => {
                // View and change the default node
                select_default_node(&server_url);
            }
            "5" => {
                // Run an AI-generated test battery
                run_ai_test(&server_url);
            }
            "6" => {
                // Exit the program
                println!("\nExiting program. Goodbye!");
                std::process::exit(0);
            }
            _ => println!("\nInvalid choice. Please enter a number between 1 and 6."),
        }
    }
}

// Function to prompt the user for a server URL
// Returns the user-provided URL or a default URL if none specified
fn get_server_url() -> String {
    print!("Enter server URL (default: http://localhost:8080): ");
    // Flush to ensure the prompt is displayed before waiting for input
    io::stdout().flush().unwrap();

    // Read user input
    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();

    // Trim whitespace and return appropriate URL
    let url = url.trim();
    if url.is_empty() {
        // Return default if nothing entered
        "http://localhost:8080".to_string()
    } else {
        url.to_string()
    }
}

// Function to collect test parameters from the user
// Returns a TestParams structure if successful, or None if the user cancels
fn collect_test_params(default_node: &str) -> Option<TestParams> {
    // Generate a unique test ID using UUID v4
    // This ensures each test has a globally unique identifier
    let id = Uuid::new_v4().to_string();

    // Get the test name from the user
    print!("Enter a name for this test: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim().to_string();
    // If no name provided, create a default name using part of the UUID
    let name = if name.is_empty() {
        format!("Test-{}", &id[0..8])
    } else {
        name
    };

    // Display test type selection menu
    println!("\nWhich test do you want to run?");
    println!("1. CPU");
    println!("2. Memory");
    println!("3. Disk");
    print!("Enter your choice (1-3): ");
    io::stdout().flush().unwrap();

    // Read test type selection
    let mut test_type = String::new();
    io::stdin().read_line(&mut test_type).unwrap();

    // Convert numeric choice to actual test type string
    let test_type = match test_type.trim() {
        "1" => "cpu",
        "2" => "mem",
        "3" => "disk",
        _ => {
            println!("\nInvalid choice. Returning to main menu.");
            return None;
        }
    };

    // Initialize TestParams with basic values
    let mut params = TestParams {
        id,
        name,
        test_type: test_type.to_string(),
        threads: None,
        duration: 0,
        load: None,
        size: None,
        fork: None,
        scheduled_time: None,
        node: default_node.to_string(),
    };

    // Note: There's a comment about adding the ability to use default node or select a custom one
    // This would be a future enhancement to let users customize the node without changing the default

    // Get test duration - common for all test types
    print!("Enter test duration (in seconds): ");
    io::stdout().flush().unwrap();
    let mut duration = String::new();
    io::stdin().read_line(&mut duration).unwrap();
    // Parse as u32 or default to 60 seconds if invalid
    params.duration = duration.trim().parse().unwrap_or(60);

    // Collect parameters specific to each test type
    match test_type {
        "cpu" => {
            // CPU test needs thread count, load percentage, and fork option
            print!("Enter number of threads: ");
            io::stdout().flush().unwrap();
            let mut threads = String::new();
            io::stdin().read_line(&mut threads).unwrap();
            params.threads = Some(threads.trim().parse().unwrap_or(1));

            print!("Enter CPU load (percentage): ");
            io::stdout().flush().unwrap();
            let mut load = String::new();
            io::stdin().read_line(&mut load).unwrap();
            params.load = Some(load.trim().parse().unwrap_or(50));

            print!("Enable fork? (y/n): ");
            io::stdout().flush().unwrap();
            let mut fork = String::new();
            io::stdin().read_line(&mut fork).unwrap();
            params.fork = Some(fork.trim().to_lowercase() == "y");
        }
        "mem" => {
            // Memory test needs thread count and memory size
            print!("Enter number of threads: ");
            io::stdout().flush().unwrap();
            let mut threads = String::new();
            io::stdin().read_line(&mut threads).unwrap();
            params.threads = Some(threads.trim().parse().unwrap_or(1));

            print!("Enter memory size (in MB): ");
            io::stdout().flush().unwrap();
            let mut size = String::new();
            io::stdin().read_line(&mut size).unwrap();
            params.size = Some(size.trim().parse().unwrap_or(100));
        }
        "disk" => {
            // Disk test needs thread count and disk size
            print!("Enter number of threads: ");
            io::stdout().flush().unwrap();
            let mut threads = String::new();
            io::stdin().read_line(&mut threads).unwrap();
            params.threads = Some(threads.trim().parse().unwrap_or(1));

            print!("Enter disk size (in MB): ");
            io::stdout().flush().unwrap();
            let mut size = String::new();
            io::stdin().read_line(&mut size).unwrap();
            params.size = Some(size.trim().parse().unwrap_or(100));
        }
        _ => unreachable!(), // This should never happen due to previous validation
    }

    // Option to schedule the test for a specific time
    print!("Schedule this test for a specific time? (y/n): ");
    io::stdout().flush().unwrap();
    let mut schedule = String::new();
    io::stdin().read_line(&mut schedule).unwrap();

    if schedule.trim().to_lowercase() == "y" {
        // Get time in HH:MM format
        print!("Enter time (HH:MM): ");
        io::stdout().flush().unwrap();
        let mut time_str = String::new();
        io::stdin().read_line(&mut time_str).unwrap();

        // Parse the input time using chrono's time parser
        if let Ok(time) = NaiveTime::parse_from_str(&time_str.trim(), "%H:%M") {
            // Get current date and time
            let now = Local::now();
            // Combine today's date with the specified time
            let mut scheduled_datetime = now.date_naive().and_time(time);
            
            // If the scheduled time has already passed today, schedule for tomorrow
            if scheduled_datetime < now.naive_local() {
                scheduled_datetime += chrono::Duration::days(1);
            }
            
            // Convert to Unix timestamp (seconds since epoch)
            let scheduled_timestamp = Local
                .from_local_datetime(&scheduled_datetime)
                .unwrap()
                .timestamp() as u64;
                
            params.scheduled_time = Some(scheduled_timestamp);
            println!(
                "\nTest scheduled for {} Returning to the main menu...",
                scheduled_datetime.format("%Y-%m-%d %H:%M")
            );
        } else {
            println!("\nInvalid time format. Test will run immediately.");
        }
    }

    // Return the completed test parameters
    Some(params)
}

// Function to display available nodes and select a default node
// Note: This function currently only displays nodes but doesn't fully implement selection
fn select_default_node(server_url: &str) {
    println!("\nFetching available nodes...");
    
    // Create a Tokio runtime for async HTTP request
    let rt = Runtime::new().unwrap();
    let nodes_response = rt.block_on(async {
        // Create HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
            
        // Send GET request to retrieve nodes
        client.get(&format!("{}/nodes", server_url))
            .send()
            .await
    });
    
    // Note: There's a comment about adding filtering capabilities for large node lists
    // This would be a future enhancement to handle systems with many nodes
    
    // Display the nodes response
    match nodes_response { 
        // Note: The comment mentions that the node format isn't ideal
        // Current format is like [{"name":"minikube"},{"name":"minikube-m02"}]
        // A future enhancement could parse and display this more neatly
        Ok(response) => {
            match rt.block_on(async { response.text().await }) {
                Ok(nodes_text) => {
                    println!("\nAvailable nodes:");
                    println!("{}", nodes_text);
                }
                Err(e) => println!("Failed to parse nodes response: {}", e),
            }
        }
        Err(e) => println!("Failed to fetch nodes: {}", e),
    }
    
    // Note: There's a comment about adding default node selection here
    // This would be a future enhancement to allow changing the default_node
    
    // Pause for user to review the nodes before returning to menu
    println!("\nPress Enter to return to the main menu...");
    let mut _pause = String::new();
    io::stdin().read_line(&mut _pause).unwrap();
}

// Function to run an AI-generated battery of stress tests
// This uses an external AI script (mogAI.py) to generate test configurations
/// Run an AI-generated battery of stress tests by invoking mogAI.py,
/// showing comments, confirming, then sending each JSON block to the server.
fn run_ai_test(server_url: &str) {
    // Generate a unique test ID for this AI test session
    let session_id = Uuid::new_v4().to_string();
    println!("\n=== AI Test Session: {} ===", &session_id[0..8]);

    // 1) Prompt user for intensity level (1-10)
    print!("Enter intensity level (1-10): ");
    io::stdout().flush().unwrap();
    let mut intensity_input = String::new();
    io::stdin().read_line(&mut intensity_input).unwrap();
    let intensity: u32 = intensity_input.trim().parse().unwrap_or(5);
    
    println!("Running mogAI.py to generate tests with intensity {}...", intensity);

    // 2) Run the mogAI.py script
    // This executes the Python script that generates test configurations
    // It passes the intensity and system info as inputs
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("(echo \"{{intensity: {}}}\" && cargo run --bin sys_info) | python3 ./src/mogAI.py", intensity)) 
        .output()
        .expect("Failed to run mogAI.py");
    
    // Process the script output
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Split output into blocks separated by double newlines
    let blocks: Vec<&str> = stdout.split("\n\n").collect();
    
    // Filter out empty blocks
    let blocks: Vec<&str> = blocks.iter()
        .filter(|&b| !b.trim().is_empty())
        .cloned()
        .collect();

    // Check if any test configurations were generated
    if blocks.is_empty() {
        println!("No test configurations generated. Returning to main menu...");
        return;
    }

    // 3) Extract comments and test configurations from each block
    let mut comments = Vec::new();
    let mut test_configs = Vec::new();

    for block in &blocks {
        // Look for comment lines (starting with #)
        if let Some(comment_line) = block.lines().find(|l| l.trim_start().starts_with('#')) {
            comments.push(comment_line.trim());
        }
        
        // Extract and parse the JSON part of the block
        let json_part: String = block.lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<&str>>()
            .join("\n");
            
        if !json_part.trim().is_empty() {
            // Attempt to parse the JSON as an AiResponse
            match serde_json::from_str::<AiResponse>(&json_part) {
                Ok(config) => test_configs.push(config),
                Err(e) => println!("Warning: Failed to parse test config: {}", e),
            }
        }
    }

    // Display generated test plan to the user
    println!("\n=== Generated Test Plan ===");
    for (i, comment) in comments.iter().enumerate() {
        println!("Test {}: {}", i + 1, comment);
    }
    
    // Check if any valid test configurations were found
    if test_configs.is_empty() {
        println!("\nNo valid test configurations found. Returning to main menu...");
        return;
    }
    
    // 4) Ask for confirmation before running tests
    print!("\nRun {} test(s)? (y/n): ", test_configs.len());
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    if !choice.trim().to_lowercase().starts_with('y') {
        println!("Test execution cancelled. Returning to main menu...");
        return;
    }

    // 5) Execute the tests using our existing run_test function
    // Create runtime and HTTP client
    let rt = Runtime::new().unwrap();
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    
    println!("\nExecuting AI-generated tests...");
    
    // Execute each test configuration
    for (i, config) in test_configs.iter().enumerate() {
        // Create test parameters from the AI response
        let test_id = Uuid::new_v4().to_string();
        let test_name = format!("AI-{}-{}", config.test_type, &test_id[0..6]);
        
        // Build test parameters
        let params = TestParams {
            id: test_id,
            name: test_name,
            test_type: config.test_type.clone(),
            threads: Some(config.threads),
            duration: config.duration,
            load: config.load,
            size: config.size,
            fork: config.fork,
            scheduled_time: None,
            node: "minikube".to_string(), // Using default node
        };
        
        // Display test progress
        println!("\nTest {}/{}: {} test (duration: {}s)", 
            i + 1, 
            test_configs.len(),
            params.test_type.to_uppercase(),
            params.duration
        );
        
        // Execute the test and wait for completion
        rt.block_on(run_test(&client, server_url, &params));
    }
    
    println!("\nAll AI tests completed. Returning to main menu...");
}

// Function to execute a test by sending an HTTP request to the stress test server
// This is an async function that handles the actual test execution
async fn run_test(client: &Client, server_url: &str, params: &TestParams) {
    println!(
        "\nStarting {} test '{}' (ID: {})...",
        params.test_type, params.name, params.id
    );

    // Prepare the request payload
    // Maps our internal TestParams to the TestRequest format expected by the API
    let request = TestRequest {
        id: params.id.clone(),
        name: params.name.clone(),
        intensity: params.threads,  // The API expects 'intensity' instead of 'threads'
        duration: params.duration,
        load: params.load,
        size: params.size,
        fork: params.fork,
        node: params.node.clone(),
    };

    // Build the endpoint URL based on test type
    let endpoint = format!("{}/{}-stress", server_url, params.test_type);
    println!("Sending request to: {}", endpoint);

    // Send the HTTP POST request with JSON payload
    match client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            // Display the JSON request that was sent
            println!("{}", serde_json::to_string_pretty(&request).unwrap());
            println!(
                "Test '{}' request sent successfully! Status: {}",
                params.name,
                response.status()
            );
            
            // Try to read and display the response body
            match response.text().await {
                Ok(text) => println!("Test '{}' response: {}", params.name, text),
                Err(e) => println!("Test '{}' failed to read response: {}", params.name, e),
            }
        }
        Err(e) => {
            // Handle request failure
            println!("Test '{}' failed to execute: {}", params.name, e);
            println!("Troubleshooting: Check if the server is running at {}", server_url);
        }
    }
}
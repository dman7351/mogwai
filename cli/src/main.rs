use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{Local, NaiveTime, TimeZone};
use reqwest::Client;
use serde::Serialize;
use uuid::Uuid;

// Structure to hold test parameters
#[derive(Debug, Clone, Serialize)]
struct TestParams {
    id: String,           // Unique identifier for the test
    name: String,         // User-provided name for the test
    test_type: String,
    threads: Option<u32>,
    duration: u32,
    load: Option<u32>,
    size: Option<u32>,
    fork: Option<bool>,
    scheduled_time: Option<u64>, // Unix timestamp for scheduling
}

#[derive(Serialize)]
struct TestRequest {
    id: String,       // Pass the ID to the server
    name: String,     // Pass the name to the server
    intensity: Option<u32>,
    duration: u32,
    load: Option<u32>,
    size: Option<u32>,
    fork: Option<bool>,
}

fn main() {
    // Multi-line welcome message
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

    // Get the server URL from the user
    let server_url = get_server_url();
    println!("\nUsing server at: {}\n", server_url);

    // Shared collection for scheduled tests
    let scheduled_tests = Arc::new(Mutex::new(Vec::<TestParams>::new()));

    // Start background thread to monitor and execute tests
    let tests_to_run = Arc::clone(&scheduled_tests);
    let server_url_clone = server_url.clone();
    let _execution_thread = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap();

            loop {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let mut tests_to_execute = Vec::new();

                // Check for tests ready to run
                {
                    let mut tests = tests_to_run.lock().unwrap();
                    let mut i = 0;
                    while i < tests.len() {
                        if let Some(scheduled_time) = tests[i].scheduled_time {
                            if current_time >= scheduled_time {
                                tests_to_execute.push(tests.remove(i));
                            } else {
                                i += 1;
                            }
                        } else {
                            tests_to_execute.push(tests.remove(i));
                        }
                    }
                }

                // Execute tests concurrently
                let mut handles = Vec::new();
                for test in tests_to_execute {
                    let client_clone = client.clone();
                    let url_clone = server_url_clone.clone();
                    let test_clone = test.clone();
                    let handle = tokio::spawn(async move {
                        run_test(&client_clone, &url_clone, &test_clone).await;
                    });
                    handles.push(handle);
                }

                // Wait for all tests to complete
                for handle in handles {
                    let _ = handle.await;
                }

                // Sleep briefly before checking again
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    });

    // Main menu loop
    loop {
        println!("\n----------------------------------------------");
        println!("Main Menu:");
        println!("1. Schedule a new test");
        println!("2. View scheduled tests");
        println!("3. Change server URL (current: {})", server_url);
        println!("4. Exit");
        print!("Enter your choice (1-4): ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                if let Some(test_params) = collect_test_params() {
                    scheduled_tests.lock().unwrap().push(test_params);
                }
            }
            "2" => {
                let tests = scheduled_tests.lock().unwrap();
                if tests.is_empty() {
                    println!("\nNo tests currently scheduled.");
                } else {
                    println!("\n=== Scheduled Tests ===");
                    for (i, test) in tests.iter().enumerate() {
                        if let Some(time) = test.scheduled_time {
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
                
                // Require user input before returning to the menu
                println!("\nPress Enter to return to the main menu...");
                let mut _pause = String::new();
                io::stdin().read_line(&mut _pause).unwrap();
            },
            "3" => {
                let new_url = get_server_url();
                println!("\nServer URL changed to: {}", new_url);
                // Note: In a real implementation, you should update the URL for the execution thread.
            }
            "4" => {
                println!("\nExiting program. Goodbye!");
                std::process::exit(0);
            }
            _ => println!("\nInvalid choice. Please enter a number between 1 and 4."),
        }
    }
}

// Prompt user for server URL
fn get_server_url() -> String {
    print!("Enter server URL (default: http://localhost:8080): ");
    io::stdout().flush().unwrap();

    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();

    let url = url.trim();
    if url.is_empty() {
        "http://localhost:8080".to_string()
    } else {
        url.to_string()
    }
}

// Collect test parameters from the user
fn collect_test_params() -> Option<TestParams> {
    // Generate a unique test ID
    let id = Uuid::new_v4().to_string();

    // Get the test name from the user
    print!("Enter a name for this test: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim().to_string();
    let name = if name.is_empty() {
        format!("Test-{}", &id[0..8])
    } else {
        name
    };

    // Select test type
    println!("\nWhich test do you want to run?");
    println!("1. CPU");
    println!("2. Memory");
    println!("3. Disk");
    print!("Enter your choice (1-3): ");
    io::stdout().flush().unwrap();

    let mut test_type = String::new();
    io::stdin().read_line(&mut test_type).unwrap();

    let test_type = match test_type.trim() {
        "1" => "cpu",
        "2" => "memory",
        "3" => "disk",
        _ => {
            println!("\nInvalid choice. Returning to main menu.");
            return None;
        }
    };

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
    };

    // Common parameter: duration
    print!("Enter test duration (in seconds): ");
    io::stdout().flush().unwrap();
    let mut duration = String::new();
    io::stdin().read_line(&mut duration).unwrap();
    params.duration = duration.trim().parse().unwrap_or(60);

    // Specific parameters based on test type
    match test_type {
        "cpu" => {
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
        "memory" => {
            print!("Enter memory size (in MB): ");
            io::stdout().flush().unwrap();
            let mut size = String::new();
            io::stdin().read_line(&mut size).unwrap();
            params.size = Some(size.trim().parse().unwrap_or(100));
        }
        "disk" => {
            print!("Enter disk size (in MB): ");
            io::stdout().flush().unwrap();
            let mut size = String::new();
            io::stdin().read_line(&mut size).unwrap();
            params.size = Some(size.trim().parse().unwrap_or(100));
        }
        _ => unreachable!(),
    }

    // Option to schedule the test for a specific time
    print!("Schedule this test for a specific time? (y/n): ");
    io::stdout().flush().unwrap();
    let mut schedule = String::new();
    io::stdin().read_line(&mut schedule).unwrap();

    if schedule.trim().to_lowercase() == "y" {
        print!("Enter time (HH:MM): ");
        io::stdout().flush().unwrap();
        let mut time_str = String::new();
        io::stdin().read_line(&mut time_str).unwrap();

        // Parse the input time
        if let Ok(time) = NaiveTime::parse_from_str(&time_str.trim(), "%H:%M") {
            let now = Local::now();
            let mut scheduled_datetime = now.date_naive().and_time(time);
            // If the scheduled time has already passed, schedule for the next day
            if scheduled_datetime < now.naive_local() {
                scheduled_datetime += chrono::Duration::days(1);
            }
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

    Some(params)
}

// Function to execute the test by sending an HTTP request
async fn run_test(client: &Client, server_url: &str, params: &TestParams) {
    println!(
        "\nStarting {} test '{}' (ID: {})...",
        params.test_type, params.name, params.id
    );

    // Prepare the request payload
    let request = TestRequest {
        id: params.id.clone(),
        name: params.name.clone(),
        intensity: params.threads,
        duration: params.duration,
        load: params.load,
        size: params.size,
        fork: params.fork,
    };

    // Build the endpoint URL
    let endpoint = format!("{}/{}-stress", server_url, params.test_type);
    println!("Sending request to: {}", endpoint);

    // Send the HTTP request
    match client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            println!(
                "Test '{}' request sent successfully! Status: {}",
                params.name,
                response.status()
            );
            match response.text().await {
                Ok(text) => println!("Test '{}' response: {}", params.name, text),
                Err(e) => println!("Test '{}' failed to read response: {}", params.name, e),
            }
        }
        Err(e) => {
            println!("Test '{}' failed to execute: {}", params.name, e);
            println!("Troubleshooting: Check if the server is running at {}", server_url);
        }
    }

    println!(
        "Finished {} test '{}' (ID: {})\nReturning to main menu...", params.test_type, params.name, params.id);
        println!("\n----------------------------------------------");
        println!("Main Menu:");
        println!("1. Schedule a new test");
        println!("2. View scheduled tests");
        println!("3. Change server URL (current: {})", server_url);
        println!("4. Exit");
        print!("Enter your choice (1-4): ");
}

/**
 * Mogwai Test GUI - Performance Testing Utility
 * CS 488 Senior Project/Final 
 * gui.rs
 * This application provides a graphical interface for running CPU, memory, and disk
 * stress tests across different environments.It allows users to configure test parameters,
 * monitor test execution, and save test results for analysis.
 *
 */
// === LIBRARY IMPORTS ===
use iced::widget::{
    toggler, Button, Checkbox, Column, Container, PickList, Row, Rule, Scrollable, Space, Text,
    TextInput,
};
use iced::{alignment, Alignment, Application, Color, Command, Element, Length, Settings, Theme};
use serde_json::{from_str as json_from_str, to_string_pretty, Value};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// === ENVIRONMENT CONFIGURATION ===
/**
 *  Defines available environments for running tests
 * - Local: Local development environment (http://localhost:8080)
 * - Kubernetes: Kubernetes cluster environment (http://localhost:8081)
 * - Custom: Custom URL environment specified by the user
 **/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Local,      // Local development environment
    Kubernetes, // Kubernetes cluster environment
    Custom,     // Custom URL environment
}

// Sets the default environment to local for new application instances.
impl Default for Environment {
    fn default() -> Self {
        Environment::Local
    }
}

// Implements display formatting for environment types, rendering the env in the UI
impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Kubernetes => write!(f, "Kubernetes"),
            Self::Custom => write!(f, "Custom URL"),
        }
    }
}

// === APPLICATION MESSAGES ===
/**
 * Message types for handling user interactions and async operations.
 * enum Message defines all possible events that can occur in the application.
 * Each variant represents a specific action/event that requires handling in the update logic.
 **/
#[derive(Debug, Clone)]
pub enum Message {
    // Test configuration messages
    ToggleTest(TestType, bool), // Message to toggle the selection of a test type
    RunPressed,                 // Message when the "Run Tests" button is pressed
    ListTasksPressed,           // Message when the "List Tasks" button is pressed
    ServerUrlChanged(String),   // Message when the server URL input field changes (new URL value)
    DurationChanged(String),    // Message when the test duration input field changes (new duration value)
    IntensityChanged(String),   // Message when the test intensity input field changes (new intensity value)
    SizeChanged(String),        // Message when the test size input field changes (new size value)
    LoadChanged(String),        // Message when the CPU load percentage input field changes (new load value)
    ForkToggled(bool),          // Message when the "Fork Test" toggle is changed (new toggle state)
    ToggleAdvanced,             // Message to toggle the visibility of advanced settings

    TestComplete(String),               // Message received when a test execution completes.
    TasksListed(String),                // Message received with the list of running tasks.
    EnvironmentSelected(Environment),   // Message when a different environment is selected from the dropdown (new environment)
    #[allow(dead_code)]
    LogsReceived(String),               // Message received containing logs from the test execution. 

    NodeStatusReceived(String),         // Message received with the status of the nodes involved in the test.
    SaveResultsPressed,                 // Message when the "Save Results" button is pressed.
    ResultsSaved(Result<(), String>),   // Message indicating the result of the save operation.
}
// === TEST TYPES ===
/**
* These represent the different performance tests that can be executed:
* - CPU
* - Memory
* - Disk
**/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestType {
    Cpu,    
    Memory, 
    Disk,   
}

// === MAIN APPLICATION STRUCT ===
/**
 * Main application state container
 * Holds all current configuration, status, and result information.
 * This struct serves as the model for the application, maintaining
 * the state of all UI elements, test configurations, and results.
 */
pub struct GuiApp {
    // Test selection and parameters
    selected_tests: Vec<TestType>, // Vector to store the currently selected test types.
    server_url: String,            // The URL of the server to send test requests to.
    environment: Environment,      // The currently selected environment.
    duration: String,              // The duration of the tests, as a string from user input.
    intensity: String,             // The intensity of the tests (e.g., number of threads).
    size: String,                  // The size parameter for memory and disk tests (in MB).
    load: String,                  // The CPU load percentage for the CPU test, as a string.
    fork: bool,                    // Flag indicating if the CPU test should fork separate processes.
    // State tracking
    status_message: Option<String>, // Message to display status updates and results to the user.
    node_status: Option<String>,    // Status information received from the test nodes.
    test_results: Option<String>,   // The raw results of the completed tests.
    show_advanced: bool,            // Flag to control the visibility of advanced settings.
    running_tests: bool,            // Flag to indicate if tests are currently running.
    last_test_id: Option<String>,   // The ID of the last run test batch, used for fetching node status.
}

// === APPLICATION IMPLEMENTATION ===
/**
 * Implementation of the Iced Application trait for the GuiApp
 * This defines the application lifecycle handling including initialization,
 * event updates, and UI rendering.
 */
impl Application for GuiApp {
    type Executor = iced::executor::Default; // Use default Iced executor for async operations.
    type Message = Message;                  // Use defined Message enum for application events.
    type Theme = Theme;                      // Use default Iced theme.
    type Flags = ();                         // initialization flags

    /**
     * Initialize the application with default settings
     * This function is called when the application starts and sets up the initial state.
     */
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            GuiApp {
                selected_tests: vec![],                             // No tests selected by default.
                server_url: String::from("http://localhost:8080"),  // Default server URL for local environment.
                environment: Environment::Local,                    // Default to local environment.
                duration: String::from("10"),                       // Default test duration: 10 seconds.
                intensity: String::from("4"),                       // Default intensity: 4 threads.
                size: String::from("256"),                          // Default size: 256 MB.
                load: String::from("70.0"),                         // Default CPU load: 70%.
                fork: false,                                        // Default: don't fork processes.
                status_message: None,                               // No status message initially.
                node_status: None,                                  // No node status initially.
                show_advanced: false,                               // Advanced settings hidden by default.
                running_tests: false,                               // No tests running initially.
                test_results: None,                                 // No test results initially.
                last_test_id: None,                                 // No last test ID initially.
            },
            Command::none(),                                        // No initial command to execute.
        )
    }

    /**
     * Set name of application window title.
     */
    fn title(&self) -> String {
        "Mogwai Test GUI".into()
    }

    /**
     * Handle all application events and update state accordingly.
     * This updates function that processes all user interactions
     * and system events, updating the application state.
     **/
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            // === INPUT FIELD CHANGES ===
            /*
             * Toggle a test type selection (checked/unchecked)
             * Adds or removes test types from the selected_tests vector
             */
            Message::ToggleTest(test, checked) => {
                if checked && !self.selected_tests.contains(&test) {
                    self.selected_tests.push(test);             // Add test if checked and not already selected.
                } else {
                    self.selected_tests.retain(|&t| t != test); // Remove test if unchecked.
                }
            }
            Message::ServerUrlChanged(url) => self.server_url = url,            // Update the server URL in the application state.
            Message::DurationChanged(duration) => self.duration = duration,     // Update the test duration in the application state.
            Message::IntensityChanged(intensity) => self.intensity = intensity, // Update the test intensity in the application state.
            Message::SizeChanged(size) => self.size = size,                     // Update the test size in the application state.
            Message::LoadChanged(load) => self.load = load,                     // Update the CPU load percentage in the application state.
            Message::ForkToggled(fork) => self.fork = fork,                     // Update the fork option in the application state.
            Message::ToggleAdvanced => self.show_advanced = !self.show_advanced, // Toggle the visibility of advanced settings.

            /*
             * Handle environment selection.
             * Updates the server URL based on the selected environment.
             */
            Message::EnvironmentSelected(env) => {
                self.environment = env;                                             // Update the selected environment in the application state.
                self.server_url = match env {
                    Environment::Local => "http://localhost:8080".to_string(),      // Set default URL for Local environment.
                    Environment::Kubernetes => "http://localhost:8081".to_string(), // Set default URL for Kubernetes environment.
                    Environment::Custom => self.server_url.clone(),                 // Keep the existing custom URL.
                };
            }

            // === TEST EXECUTION & RESULTS ===
            /*
             * Handle test completion.
             * Updates the UI with results and requests node status.
             */
            Message::TestComplete(results) => {
                self.running_tests = false;                     // Reset the running tests flag.
                self.status_message = Some(results.clone());    // Update the status message with the test results.
                self.test_results = Some(results);              // Store the test results in the application state.

                // Fetch node status as needed
                if let Some(test_id) = &self.last_test_id {
                    return fetch_node_status(self.server_url.clone(), test_id.clone());
                }
            }
            /*
             * Process node status updates.
             * Displays node status information in the UI.
             */
            Message::NodeStatusReceived(status) => {
                self.node_status = Some(status); // Update the displayed node status.
            }
            /*
             * Process test logs.
             * Appends logs to the node status display.
             */
            Message::LogsReceived(logs) => {
                if let Some(existing) = &self.node_status {
                    self.node_status = Some(format!("{}\n\nLogs:\n{}", existing, logs));
                } else {
                    self.node_status = Some(format!("Logs:\n{}", logs));
                }
            }

            // === Actions ===
            /*
             * Handle save results button press.
             * Initiates saving test results to a file.
             */
            Message::SaveResultsPressed => {
                if let Some(results) = &self.test_results {
                    return save_results(results.clone()); // Save results to file.
                }
            }
            /*
             * Process result saving completion.
             * Updates status message with success or failure.
             */
            Message::ResultsSaved(result) => match result {
                Ok(_) => {
                    self.status_message = Some(format!(
                        "{}\n\nResults successfully saved to results directory.",
                        self.status_message.clone().unwrap_or_default()
                    )); // Update the status message on successful saving of results.
                }
                Err(e) => {
                    self.status_message = Some(format!(
                        "{}\n\nFailed to save results: {}",
                        self.status_message.clone().unwrap_or_default(),
                        e
                    )); // Update status on save failure.
                }
            },
            /*
             * Process task listing results.
             * Updates status with the list of running tasks.
             */
            Message::TasksListed(results) => {
                self.status_message = Some(results);
            } // Update status with the list of tasks.

            /*
             * Handle list tasks button press.
             * Requests a list of all running test tasks.
             */
            Message::ListTasksPressed => {
                self.status_message = Some("Fetching running tasks...".to_string());
                return list_tasks(self.server_url.clone());
            }
            /*
             * Handle run button press.
             * Validates test configuration and initiates test execution.
             */
            Message::RunPressed => {
                // Validation: ensure tests are selected.
                if self.selected_tests.is_empty() {
                    self.status_message = Some("No tests selected.".to_string());
                    return Command::none();
                }
                // Validation: fork option requires CPU test.
                if self.fork && !self.selected_tests.contains(&TestType::Cpu) {
                    self.status_message =
                        Some("Fork option requires CPU test to be selected.".to_string());
                    return Command::none();
                }

                // Update state to reflect running tests.
                self.running_tests = true;
                self.status_message = Some("Running tests...".to_string());

                // Generate unique batch ID for this test run.
                let batch_id = Uuid::new_v4().to_string();
                self.last_test_id = Some(batch_id.clone());

                // Execute tests asynchronously and return command.
                return Command::perform(
                    execute_tests(
                        self.selected_tests.clone(),
                        self.server_url.clone(),
                        batch_id,
                        self.duration.clone(),
                        self.intensity.clone(),
                        self.size.clone(),
                        self.load.clone(),
                        self.fork,
                    ),
                    Message::TestComplete, // Message to send when tests complete.
                );
            }
        }
        Command::none() // Default case: no command to execute.
    }

    /*
     * Render application UI.
     * Builds the complete user interface layout based on the current application state.
     * This function defines all UI elements, their arrangement, and how they connect
     * to application events.
     */
    fn view(&self) -> Element<'_, Self::Message> {
        // === HEADER SECTION ===
        // Main application title and subtitle.
        let header = Column::new()
            .push(
                Text::new("Mogwai Stress Tool")
                    .size(32)
                    .style(Color::from_rgb(0.3, 0.4, 0.5)),
            )
            .push(
                Text::new("Performance Test Utility")
                    .size(18)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
            )
            .spacing(5)
            .width(Length::Fill)
            .align_items(Alignment::Center);

        // Horizontal rule to separate sections.
        let separator = Rule::horizontal(1);

        // === ADVANCED SETTINGS TOGGLE ===
        // Toggle control for showing/hiding advanced settings.
        let advanced_toggle = Row::new()
            .push(Text::new("Advanced Settings").size(16))
            .push(Space::with_width(Length::Fill))
            .push(
                toggler(None, self.show_advanced, |_| Message::ToggleAdvanced)
                    .width(Length::Fixed(40.0)),
            )
            .width(Length::Fill)
            .align_items(Alignment::Center);

        // === ADVANCED SETTINGS SECTION ===
        // Conditionally rendered based on toggle state.
        let advanced_section = if self.show_advanced {
            Column::new()
                .push(
                    Row::new()
                        .push(Text::new("Environment:").width(Length::FillPortion(1)))
                        .push(
                            PickList::new(
                                &[
                                    Environment::Local,
                                    Environment::Kubernetes,
                                    Environment::Custom,
                                ],
                                Some(self.environment),
                                Message::EnvironmentSelected,
                            )
                            .width(Length::FillPortion(2)),
                        )
                        .spacing(10)
                        .align_items(Alignment::Center),
                )
                .push(
                    TextInput::new("Server URL (e.g., http://localhost:8080)", &self.server_url)
                        .on_input(Message::ServerUrlChanged)
                        .padding(10),
                )
                .spacing(10)
                .width(Length::Fill)
        } else {
            Column::new() // Empty column when advanced settings are hidden.
        };

        // === TEST SELECTION SECTION ===
        // Checkboxes for selecting which tests to run.
        let checkboxes = Column::new()
            .push(Text::new("Select Tests:").size(18))
            .push(
                Row::new()
                    .push(
                        Container::new(Checkbox::new(
                            "CPU Test",
                            self.selected_tests.contains(&TestType::Cpu),
                            move |checked| Message::ToggleTest(TestType::Cpu, checked),
                        ))
                        .width(Length::FillPortion(1)),
                    )
                    .push(
                        Container::new(Checkbox::new(
                            "Memory Test",
                            self.selected_tests.contains(&TestType::Memory),
                            move |checked| Message::ToggleTest(TestType::Memory, checked),
                        ))
                        .width(Length::FillPortion(1)),
                    )
                    .push(
                        Container::new(Checkbox::new(
                            "Disk Test",
                            self.selected_tests.contains(&TestType::Disk),
                            move |checked| Message::ToggleTest(TestType::Disk, checked),
                        ))
                        .width(Length::FillPortion(1)),
                    )
                    .spacing(10),
            )
            .spacing(10)
            .width(Length::Fill);

        // === TEST PARAMETERS SECTION ===
        // Input fields for configuring test parameters.
        let params_title = Text::new("Test Parameters:").size(18);

        // First row of parameter inputs: Duration and Intensity.
        let row1 = Row::new()
            .push(
                Container::new(
                    TextInput::new("Duration (seconds)", &self.duration)
                        .on_input(Message::DurationChanged)
                        .padding(8),
                )
                .width(Length::Fill),
            )
            .push(
                Container::new(
                    TextInput::new("Intensity (threads)", &self.intensity)
                        .on_input(Message::IntensityChanged)
                        .padding(8),
                )
                .width(Length::Fill),
            )
            .spacing(10)
            .width(Length::Fill);

        // Second row of parameter inputs: Size and CPU Load.
        let row2 = Row::new()
            .push(
                Container::new(
                    TextInput::new("Size (MB)", &self.size)
                        .on_input(Message::SizeChanged)
                        .padding(8),
                )
                .width(Length::Fill),
            )
            .push(
                Container::new(
                    TextInput::new("CPU Load (%)", &self.load)
                        .on_input(Message::LoadChanged)
                        .padding(8),
                )
                .width(Length::Fill),
            )
            .spacing(10)
            .width(Length::Fill);

        // === CPU OPTIONS SECTION ===
        // Additional options specific to CPU tests.
        let fork_section = Column::new()
            .push(Text::new("CPU Test Options:").size(18))
            .push(
                Container::new(Checkbox::new("Fork Test", self.fork, Message::ForkToggled))
                    .padding(5),
            )
            .spacing(5)
            .width(Length::Fill);

        // === HELP INFORMATION SECTION ===
        // Explanatory text about test parameters.
        let helper_text = Container::new(
            Column::new()
                .push(
                    Text::new("Test Parameter Information:")
                        .size(16)
                        .style(Color::from_rgb(0.3, 0.4, 0.5)),
                )
                .push(Text::new(
                    "• CPU Test: Uses intensity (threads), duration, and load percentage",
                ))
                .push(Text::new(
                    "  - With Fork enabled: Uses separate processes instead of threads",
                ))
                .push(Text::new(
                    "• Memory Test: Uses intensity (threads), size (MB), and duration",
                ))
                .push(Text::new(
                    "• Disk Test: Uses intensity (threads), size (MB), and duration",
                ))
                .spacing(5),
        )
        .style(iced::theme::Container::Box)
        .padding(10)
        .width(Length::Fill);

        // === ACTION BUTTONS ===
        // "Run Tests" button (changes state when tests are running).
        let run_button = if self.running_tests {
            Button::new(
                Text::new("RUNNING...")
                    .size(18)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .padding([12, 30])
            .style(iced::theme::Button::Secondary)
            .width(Length::Fill)
        } else {
            Button::new(
                Text::new("RUN TESTS")
                    .size(18)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .on_press(Message::RunPressed)
            .padding([12, 30])
            .style(iced::theme::Button::Primary)
            .width(Length::Fill)
        };

        // "List Tasks" button.
        let list_tasks_button = Button::new(
            Text::new("LIST TASKS")
                .size(16)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .on_press(Message::ListTasksPressed)
        .padding([8, 20])
        .style(iced::theme::Button::Secondary)
        .width(Length::Fill);

        // Save results button.
        let save_button = Button::new(
            Text::new("SAVE RESULTS")
                .size(16)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .on_press(Message::SaveResultsPressed)
        .padding([8, 20])
        .style(iced::theme::Button::Secondary)
        .width(Length::Fill);

        // === BUTTON LAYOUTS ===
        // Run and List Tasks buttons.
        let primary_button_row = Row::new()
            .push(Container::new(run_button).width(Length::FillPortion(2)))
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(Container::new(list_tasks_button).width(Length::FillPortion(1)))
            .spacing(10)
            .width(Length::Fixed(450.0));

        // Row containing Save Results button.
        let secondary_button_row = Row::new()
            .push(Container::new(save_button).width(Length::Fill))
            .spacing(10)
            .width(Length::Fixed(450.0));

        // === RESULTS DISPLAY SECTION ===
        // Display area for test results and status messages.
        let test_results_view = Container::new(
            Column::new()
                .push(
                    Text::new("Test Results:")
                        .size(18)
                        .style(Color::from_rgb(0.3, 0.4, 0.5)),
                )
                .push(
                    Container::new(
                        Scrollable::new(
                            Text::new(
                                self.status_message
                                    .clone()
                                    .unwrap_or_else(|| "No test results yet.".to_string()),
                            )
                            .size(14),
                        )
                        .height(Length::Fixed(400.0)),
                    )
                    .style(iced::theme::Container::Box)
                    .padding(10)
                    .width(Length::Fill),
                )
                .spacing(10),
        )
        .width(Length::Fill);

        // === MAIN LAYOUT ===
        // Assemble UI sections into layout.
        let content = Column::new()
            .push(header)
            .push(separator)
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(advanced_toggle)
            .push(advanced_section)
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(checkboxes)
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(params_title)
            .push(row1)
            .push(row2)
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(fork_section)
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(helper_text)
            .push(Space::with_height(Length::Fixed(15.0)))
            .push(Container::new(primary_button_row).center_x())
            .push(Space::with_height(Length::Fixed(10.0)))
            .push(Container::new(secondary_button_row).center_x())
            .push(Space::with_height(Length::Fixed(15.0)))
            .push(test_results_view)
            .spacing(8)
            .width(Length::Fill);

        // Wrap content in scrollable container and return as Element.
        Container::new(Scrollable::new(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .padding(30)
            .into()
    }

    /**
     * Set up any event subscriptions (none used in this application).
     */
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}

// === HELPER FUNCTIONS ===
/*
 * Fetch node status for a test
 * This function makes an API request to retrieve the status of nodes
 * that were involved in a specific test run.
 */
fn fetch_node_status(server_url: String, test_id: String) -> Command<Message> {
    Command::perform(
        async move {
            // Sleep to give the test time to propagate to nodes.
            // This delay ensures that node status data is available after test completion.
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            // Construct the API endpoint URL for node status.
            let endpoint = format!("{}/nodes/{}", server_url, test_id);
            println!("Fetching node status from: {}", endpoint);

            // Use curl to make the API request.
            let command = format!("curl -X GET {}", endpoint);
            let output = ProcessCommand::new("sh").arg("-c").arg(&command).output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);

                        if stdout.trim().is_empty() {
                            // Handle empty response.
                            "No node status available.".to_string()
                        } else {
                            // Try to parse response as JSON for better formatting.
                            match json_from_str::<Value>(&stdout) {
                                Ok(json) => {
                                    // Check if the API returned a "Not Found" error.
                                    if let Some(detail) = json.get("detail") {
                                        if detail.as_str() == Some("Not Found") {
                                            format!("Node Status for Test {}:\n\nNo detailed node status available.", test_id)
                                        } else {
                                            // Format the node status for display.
                                            format!(
                                                "Node Status for Test {}:\n\n{}",
                                                test_id,
                                                format_node_status(&stdout)
                                            )
                                        }
                                    } else {
                                        // Format the node status for display.
                                        format!(
                                            "Node Status for Test {}:\n\n{}",
                                            test_id,
                                            format_node_status(&stdout)
                                        )
                                    }
                                }
                                //Handle non-JSON response.
                                Err(_) => format!("Node Status for Test {}:\n{}", test_id, stdout), 
                            }
                        }
                    } else {
                        // Handle unsuccessful HTTP status
                        "Failed to fetch node status.".to_string()
                    }
                }
                // Handle connection errors.
                Err(_) => "Error connecting to server for node status.".to_string(),
            }
        },
        Message::NodeStatusReceived, // Message to send the result.
    )
}

/*
 * Save test results to a file
 * This function creates a timestamped file in the 'results' directory
 * and saves the raw test results string to it.
 */
fn save_results(results: String) -> Command<Message> {
    Command::perform(
        async move {
            // Create results directory if it doesn't exist.
            let results_dir = Path::new("results");
            if !results_dir.exists() {
                if let Err(e) = fs::create_dir_all(results_dir) {
                    return Err(format!("Failed to create results directory: {}", e));
                }
            }

            // Generate filename with Unix timestamp.
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let filename = format!("mogwai_results_{}.txt", timestamp);
            let path = results_dir.join(filename);

            // Write results to file, handling potential errors.
            match File::create(&path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(results.as_bytes()) {
                        return Err(format!("Failed to write to file: {}", e));
                    }
                    Ok(()) // Return success.
                }
                Err(e) => Err(format!("Failed to create file: {}", e)),
            }
        },
        Message::ResultsSaved, // Message to send with the result.
    )
}

/*
 * List running tasks on the server
 * This function makes an API request to retrieve all currently running
 * test tasks and formats them for display.
 */
fn list_tasks(server_url: String) -> Command<Message> {
    Command::perform(
        async move {
            // Construct the API endpoint URL for task listing.
            let endpoint = format!("{}/tasks", server_url);
            println!("Fetching tasks from: {}", endpoint);

            // Use curl to make the API request.
            let command = format!("curl -X GET {}", endpoint);
            let output = ProcessCommand::new("sh").arg("-c").arg(&command).output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.trim().is_empty() {
                            // Handle empty response.
                            "No running tasks found.".to_string()
                        } else {
                            // Parse and format the task list for display.
                            parse_tasks_response(&stdout)
                        }
                    } else {
                        // Handle unsuccessful HTTP status with error message.
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        format!("Failed to get tasks: {}", stderr)
                    }
                }
                // Handle connection errors.
                Err(e) => format!("Error fetching tasks: {}", e),
            }
        },
        Message::TasksListed, // Message to send with the result.
    )
}

/*
 * Format node status JSON into readable text
 * This function parses a JSON string containing node status information
 * and formats it into a human-readable multi-line string.
 */
fn format_node_status(json_str: &str) -> String {
    match json_from_str::<Value>(json_str) {
        Ok(json) => {
            // Check for API error responses.
            if let Some(detail) = json.get("detail") {
                if detail.as_str() == Some("Not Found") {
                    return "Node status data not found for this test ID.".to_string();
                }
            }

            let mut result = String::new();

            // Extract and format basic test info.
            if let Some(test_id) = json.get("test_id") {
                result.push_str(&format!(
                    "Test ID: {}\n",
                    test_id.as_str().unwrap_or("Unknown")
                ));
            }

            if let Some(batch_id) = json.get("batch_id") {
                result.push_str(&format!(
                    "Batch ID: {}\n",
                    batch_id.as_str().unwrap_or("Unknown")
                ));
            }

            if let Some(status) = json.get("status") {
                result.push_str(&format!(
                    "Status: {}\n",
                    status.as_str().unwrap_or("Unknown")
                ));
            }

            // Format and add sections for node info and metrics.
            format_json_section(
                &mut result,
                json.get("node_info"),
                "\n--- Node Information ---\n",
            );
            format_json_section(
                &mut result,
                json.get("metrics"),
                "\n--- Performance Metrics ---\n",
            );

            // If we couldn't extract any structured data, return the raw JSON.
            if result.is_empty() {
                return format!("Raw node status data:\n{}", json_str);
            }

            result
        }
        // Handle invalid JSON response.
        Err(_) => format!("Raw response (not valid JSON):\n{}", json_str),
    }
}

/*
 * Helper to format a JSON section in node status output
 * This function formats a specific section of the node status JSON
 * into a readable text format with appropriate indentation.
 */
fn format_json_section(result: &mut String, section: Option<&Value>, header: &str) {
    if let Some(section_value) = section {
        if let Some(obj) = section_value.as_object() {
            if !obj.is_empty() {
                // Add the section header.
                result.push_str(header);
                // Process each key-value pair in the section.
                for (key, value) in obj {
                    if value.is_object() {
                        // Handle nested objects with additional indentation.
                        result.push_str(&format!("{}:\n", key));
                        if let Some(inner_obj) = value.as_object() {
                            for (inner_key, inner_value) in inner_obj {
                                let display_value = format_json_value(inner_value);
                                result.push_str(&format!("  {}: {}\n", inner_key, display_value));
                            }
                        }
                    } else {
                        // Format simple key-value pairs.
                        let display_value = format_json_value(value);
                        result.push_str(&format!("{}: {}\n", key, display_value));
                    }
                }
            }
        }
    }
}

/*
 * Format a JSON value as a string
 * This function converts different JSON value types into appropriate
 * string representations for display.
 */
fn format_json_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),     // Use string as-is.
        Value::Number(n) => n.to_string(), // Convert number to string.
        Value::Bool(b) => b.to_string(),   // Convert boolean to "true" or "false".
        _ => value.to_string(),            // Default conversion for other types.
    }
}

/*
 * Parse and format task list response
 * This function processes a JSON response containing task information
 * and formats it into a human-readable task list.
 */
fn parse_tasks_response(stdout: &str) -> String {
    match json_from_str::<Value>(stdout) {
        Ok(json) => {
            // Check for API error responses.
            if let Some(detail) = json.get("detail") {
                if detail.as_str() == Some("Not Found") {
                    return "=== RUNNING TASKS ===\n\nNo task list available.".to_string();
                }
            } else if stdout.contains("[") && stdout.contains("]") {
                // It a JSON array, try to parse tasks.
                let mut result = String::from("=== RUNNING TASKS ===\n\n");

                // Try to parse as an array of tasks.
                if let Some(tasks) = json.as_array() {
                    if tasks.is_empty() {
                        result.push_str("No running tasks found.\n");
                    } else {
                        // Show task count and format each task.
                        result.push_str(&format!("Found {} running tasks:\n\n", tasks.len()));

                        for (i, task) in tasks.iter().enumerate() {
                            format_task_item(&mut result, i, task);
                        }
                    }
                    return result;
                }
            }

            // Default: show raw JSON if we couldn't parse it as expected.
            format!("=== RUNNING TASKS ===\n\n{}", stdout)
        }
        // Handle invalid JSON response.
        Err(_) => format!("=== RUNNING TASKS ===\n\n{}", stdout),
    }
}

/*
 * Format a single task item in the task list
 * This function formats an individual task entry from the task list
 * into a readable format with appropriate bullet points.
 */
fn format_task_item(result: &mut String, index: usize, task: &Value) {
    if let Some(task_obj) = task.as_object() {
        // Format task object with bullet points for each property.
        result.push_str(&format!("Task #{}: \n", index + 1));
        for (key, value) in task_obj {
            let display_value = format_json_value(value);
            result.push_str(&format!("  • {}: {}\n", key, display_value));
        }
        result.push_str("\n");
    } else if let Some(task_str) = task.as_str() {
        // Handle case where the task is a simple string.
        result.push_str(&format!("Task #{}: {}\n", index + 1, task_str));
    } else {
        // Default case for other formats.
        result.push_str(&format!("Task #{}: {}\n", index + 1, task.to_string()));
    }
}

/*
 * Get system information for test reports
 * This function collects various system information metrics including:
 * - Operating system name and version
 * - CPU model and core count
 * - Total system memory
 */
fn get_system_info() -> String {
    let mut info = Vec::new();

    // Try to get OS information (Linux).
    #[cfg(target_os = "linux")]
    {
        // Extract OS name from /etc/os-release file.
        if let Ok(output) = ProcessCommand::new("sh")
            .arg("-c")
            .arg("cat /etc/os-release | grep PRETTY_NAME | cut -d= -f2")
            .output()
        {
            let os_name = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('"') 
                .to_string();
            if !os_name.is_empty() {
                info.push(format!("OS: {}", os_name));
            }
        }
    }

    // Get CPU and memory information (Linux).
    #[cfg(target_os = "linux")]
    {
        // Extract CPU model information from /proc/cpuinfo.
        if let Ok(output) = ProcessCommand::new("sh")
            .arg("-c")
            .arg("cat /proc/cpuinfo | grep 'model name' | head -1 | cut -d: -f2")
            .output()
        {
            let cpu_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !cpu_info.is_empty() {
                info.push(format!("CPU: {}", cpu_info));
            }
        }

        // Get CPU core count using nproc command.
        if let Ok(output) = ProcessCommand::new("sh").arg("-c").arg("nproc").output() {
            let cpu_count = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !cpu_count.is_empty() {
                info.push(format!("CPU Cores: {}", cpu_count));
            }
        }

        // Get total memory in MB using free command.
        if let Ok(output) = ProcessCommand::new("sh")
            .arg("-c")
            .arg("free -m | grep Mem | awk '{print $2}'")
            .output()
        {
            let mem_total = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !mem_total.is_empty() {
                info.push(format!("Total Memory: {} MB", mem_total));
            }
        }
    }

    // Fallback message if no information could be gathered.
    if info.is_empty() {
        return "System information not available.".to_string();
    }

    // Join all information lines into a single string.
    info.join("\n")
}

/*
 * Get memory information from the system.
 * This function retrieves the total and used memory of the system
 * in MB. 
 */
fn get_memory_info() -> Option<(u64, u64)> {
    #[cfg(target_os = "linux")]
    {
        // Try to get total memory in MB.
        if let Ok(output) = ProcessCommand::new("sh")
            .arg("-c")
            .arg("free -m | grep Mem | awk '{print $2}'")
            .output()
        {
            let total_mem = String::from_utf8_lossy(&output.stdout).trim().to_string();

            // Try to get used memory in MB.
            if let Ok(output) = ProcessCommand::new("sh")
                .arg("-c")
                .arg("free -m | grep Mem | awk '{print $3}'")
                .output()
            {
                let used_mem = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // Parse the memory values as u64 integers.
                if !total_mem.is_empty() && !used_mem.is_empty() {
                    if let (Ok(total), Ok(used)) =
                        (total_mem.parse::<u64>(), used_mem.parse::<u64>())
                    {
                        return Some((total, used));
                    }
                }
            }
        }
    }
    // Return None if we couldn't get the memory information.
    None
}

/*
 * Execute tests with full metrics and reporting
 * This is the main asynchronous function that executes the selected tests,
 * collects results, and generates a comprehensive test report. It handles
 * the entire test lifecycle including:
 * - Report header generation
 * - System information collection
 * - Test execution for each selected test type
 * - API communication with the test server
 * - Results collection and formatting
 * - Summary generation
 */
async fn execute_tests(
    selected_tests: Vec<TestType>,      // Vector of test types to execute (CPU, Memory, Disk).
    server_url: String,                 // Base URL of the test server API.
    batch_id: String,                   // Unique ID for this batch of tests.
    duration: String,                   // Test duration in seconds.
    intensity: String,                  // Test intensity (threads).
    size: String,                       // Test size in MB (Memory & Disk tests).
    load: String,                       // CPU load percentage.
    fork: bool,                         // True/False fork processes.
) -> String {
    // Initialize results vector to store report lines.
    let mut results = Vec::new();

    // === REPORT HEADER SECTION ===
    // Add batch ID and timestamp information.
    add_report_header(&mut results, &batch_id);

    // === SYSTEM INFORMATION SECTION ===
    // Add details about the system running the tests.
    results.push(format!("SYSTEM INFORMATION"));
    results.push(format!("------------------------------------"));
    results.push(get_system_info());
    results.push(format!(""));

    // === TEST EXECUTION SECTION ===
    // Process each selected test type in sequence.
    for test in &selected_tests {
        // Add formatted header for this specific test.
        let test_name = get_test_name(test);
        add_test_header(&mut results, test_name);

        // Generate a unique ID for this test and prepare API payload.
        let test_id = Uuid::new_v4().to_string();
        let (endpoint, payload) = prepare_test_payload(
            test, &test_id, &batch_id, &duration, &intensity, &size, &load, fork,
        );

        // Add details about the API request for debugging.
        add_request_details(&mut results, &server_url, endpoint, &test_id);

        // Add information about the test parameters being used.
        add_test_parameters(
            &mut results,
            test,
            &duration,
            &intensity,
            &size,
            &load,
            fork,
        );

        // Add the raw JSON payload for reference.
        results.push(format!(""));
        results.push(format!("JSON Payload:"));
        results.push(format!("{}", payload));

        // === TEST EXECUTION ===
        // Construct the curl command to execute the test.
        let command = format!(
            "curl -X POST {}/{} -H \"Content-Type:application/json\" -d '{}'",
            server_url, endpoint, payload
        );

        // Execute the command and process the response.
        let output = ProcessCommand::new("sh").arg("-c").arg(&command).output();
        process_test_response(&mut results, output);

        // === WAIT FOR TEST COMPLETION ===
        // Indicate test is finishing.
        results.push(format!(""));
        results.push(format!(
            "Test {} started, waiting for completion...",
            test_name
        ));

        // Calculate wait time based on test duration and add buffer.
        let wait_time = calculate_wait_time(&duration);
        tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;

        // === CHECK TEST RESULTS ===
        // Query the API for test status and results.
        check_test_status(&mut results, test, &server_url, &test_id).await;

        // Mark test as completed.
        results.push(format!(""));
        results.push(format!("Test {} completed.", test_name));
        results.push(format!(""));
    }

    // === SUMMARY SECTION ===
    // overall summary of all tests in the batch.
    add_summary_section(&mut results, &batch_id, &selected_tests);

    // Join all result lines into a single string and return.
    results.join("\n")
}

/*
 * Add report header to results.
 * This function adds a header to the test report, including:
 * - Title banner.
 * - Current date and time.
 * - Unique batch ID for tracking.
 */
fn add_report_header(results: &mut Vec<String>, batch_id: &str) {
    results.push(format!("===================================="));
    results.push(format!("MOGWAI PERFORMANCE TEST REPORT"));
    results.push(format!("===================================="));
    results.push(format!(
        "Date/Time: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S") // Format current time as YYYY-MM-DD HH:MM:SS.
    ));
    results.push(format!("Batch ID: {}", batch_id));
    results.push(format!(""));
}

/*
 * Get user-friendly test name
 * Converts TestType enum values into human-readable string names.
 */
fn get_test_name(test: &TestType) -> &'static str {
    match test {
        TestType::Cpu => "CPU",
        TestType::Memory => "Memory",
        TestType::Disk => "Disk",
    }
}

/*
 * Add test header to results
 * Inserts a section header for a specific test type in the report.
 */
fn add_test_header(results: &mut Vec<String>, test_name: &str) {
    results.push(format!("===================================="));
    results.push(format!("RUNNING {} TEST", test_name));
    results.push(format!("===================================="));
}

/*
 * Prepare payload for test API request
 * Creates the appropriate API endpoint and JSON payload for the
 * selected test type with all necessary parameters.
 */
fn prepare_test_payload(
    test: &TestType,        // Reference to the TestType being executed.
    test_id: &str,          // Unique identifier for this specific test.
    batch_id: &str,         // Identifier for the batch id.
    duration: &str,         // Test duration in seconds.
    intensity: &str,        // Test intensity (number of threads).
    size: &str,             // Test size in MB (Memory & Disk tests).
    load: &str,             // CPU load percentage.
    fork: bool,             // True/False process forking.
) -> (&'static str, String) {
    // Determine the appropriate API endpoint based on test type
    let endpoint = match test {
        TestType::Cpu => "cpu-stress",    // CPU stress test endpoint.
        TestType::Memory => "mem-stress", // Memory stress test endpoint.
        TestType::Disk => "disk-stress",  // Disk stress test endpoint.
    };

    // Create the appropriate JSON payload based on test type.
    let payload = match test {
        TestType::Cpu => {
            // CPU test requires intensity, duration, load, and fork parameters.
            format!(
                r#"{{"id": "{}", "batch_id": "{}", "name": "GUI Test", "intensity": {}, "duration": {}, "load": {}, "fork": {}}}"#,
                test_id,
                batch_id,
                intensity,
                duration,
                load,
                if fork { "true" } else { "false" }
            )
        }
        TestType::Memory | TestType::Disk => {
            // Memory and Disk tests require intensity, duration, and size parameters.
            format!(
                r#"{{"id": "{}", "batch_id": "{}", "name": "GUI Test", "intensity": {}, "duration": {}, "size": {}}}"#,
                test_id, batch_id, intensity, duration, size
            )
        }
    };

    (endpoint, payload)
}

/*
 * Add request details to results
 * Adds information about the API request being made to the report,
 * including the endpoint URL and test ID.
 */
fn add_request_details(results: &mut Vec<String>, server_url: &str, endpoint: &str, test_id: &str) {
    results.push(format!("Request Details:"));
    results.push(format!("  Endpoint: {}/{}", server_url, endpoint));
    results.push(format!("  Test ID: {}", test_id));
}

/*
 * Add test parameters to results
 *
 * Adds detailed information about test parameters to the report,
 * with specific details for each test type. This includes:
 * - Basic parameters (threads, duration, etc.)
 * - Calculated values (total memory, disk usage)
 * - Technical details about test execution
 * - System resource information
 */
fn add_test_parameters(
    results: &mut Vec<String>,  // Vector to append parameter information to.
    test: &TestType,            // Reference to the TestType being executed.
    duration: &str,             // Test duration in seconds.
    intensity: &str,            // Test intensity (number of threads).
    size: &str,                 // Test size in MB (Memory & Disk tests)
    load: &str,                 // CPU load percentage.
    fork: bool,                 // True/False process forking.
) {
    results.push(format!("Test Parameters:"));

    match test {
        TestType::Cpu => {
            // === CPU TEST PARAMETERS ===
            results.push(format!("  • Threads: {}", intensity));
            results.push(format!("  • Duration: {} seconds", duration));
            results.push(format!("  • Target CPU Load: {}%", load));

            // Add fork-specific information for CPU tests.
            if fork {
                // Fork mode uses separate processes instead of threads.
                results.push(format!("  • Fork Mode: Enabled (using separate processes)"));
                results.push(format!("  • Process Count: {} processes", intensity));

                // Add technical explanation of fork mode.
                results.push(format!("  • Fork Mode Details:"));
                results.push(format!("    - Each process runs independently"));
                results.push(format!("    - Parent process monitors child processes"));
                results.push(format!(
                    "    - System resources allocated separately for each process"
                ));
            } else {
                // Thread mode uses standard threading.
                results.push(format!("  • Fork Mode: Disabled (using threads)"));

                // Add CPU cycle explanation for load percentages less than 100%.
                if let Ok(load_val) = load.parse::<f64>() {
                    if load_val < 100.0 {
                        // Calculate work and sleep times for the desired load percentage.
                        let cycle_time = 100; // ms
                        let work_time = (cycle_time as f64 * load_val / 100.0) as u64;
                        let sleep_time = cycle_time - work_time;

                        results.push(format!("  • CPU Cycle Details:"));
                        results.push(format!(
                            "    - Work period: {} ms per 100ms cycle",
                            work_time
                        ));
                        results.push(format!(
                            "    - Sleep period: {} ms per 100ms cycle",
                            sleep_time
                        ));
                    } else {
                        // 100% load means continuous work with no sleep periods.
                        results.push(format!(
                            "  • CPU Cycle Details: Running at full capacity (100% busy loop)"
                        ));
                    }
                }
            }
        }
        TestType::Memory => {
            // === MEMORY TEST PARAMETERS ===
            results.push(format!("  • Threads: {}", intensity));
            results.push(format!("  • Duration: {} seconds", duration));
            results.push(format!("  • Size per Thread: {} MB", size));

            // Calculate and add total memory allocation across all threads.
            if let (Ok(threads), Ok(size_mb)) = (intensity.parse::<usize>(), size.parse::<usize>())
            {
                let total_mb = threads * size_mb;
                results.push(format!("  • Total Memory Allocation: {} MB", total_mb));

                // Add details about memory test execution.
                results.push(format!("  • Memory Test Details:"));
                results.push(format!("    - Each thread allocates blocks of memory"));
                results.push(format!(
                    "    - Memory is actively used to prevent optimization"
                ));
                results.push(format!("    - 4KB page size access pattern"));
            }

            // Get and add current system memory information.
            let initial_memory = get_memory_info();
            results.push(format!("  • System Memory Information (Pre-Test):"));
            if let Some((total, used)) = initial_memory {
                results.push(format!("    - Total Memory: {} MB", total));
                results.push(format!("    - Used Memory: {} MB", used));
                results.push(format!("    - Free Memory: {} MB", total - used));
            } else {
                results.push(format!("    - Memory information not available"));
            }
        }
        TestType::Disk => {
            // === DISK TEST PARAMETERS ===
            results.push(format!("  • Threads: {}", intensity));
            results.push(format!("  • Duration: {} seconds", duration));
            results.push(format!("  • File Size: {} MB", size));

            // Calculate and add total disk usage across all threads.
            if let (Ok(threads), Ok(size_mb)) = (intensity.parse::<usize>(), size.parse::<usize>())
            {
                let total_mb = threads * size_mb;
                results.push(format!("  • Total Disk Usage: {} MB", total_mb));

                // Add technical details about disk test execution.
                results.push(format!("  • Disk Test Details:"));
                results.push(format!("    - Each thread creates a separate file"));
                results.push(format!("    - Alternating write and read phases"));
                results.push(format!("    - Files are cleaned up after test"));
                results.push(format!("    - Sequential I/O pattern"));
            }
        }
    }
}

/*
 * Process test response from API
 * This function handles the API response after a test has been initiated,
 * parsing and formatting the output for the test report. It:
 * - Indicates whether the request was successful
 * - Formats any JSON response for readability
 * - Includes any error details if the request failed
 */
fn process_test_response(
    results: &mut Vec<String>,
    output: Result<std::process::Output, std::io::Error>,
) {
    match output {
        Ok(output) => {
            // Determine if the HTTP request was successful.
            let status_str = if output.status.success() {
                "SUCCESS"
            } else {
                "FAILED"
            };
            results.push(format!(""));
            results.push(format!("Execution Status: {}", status_str));

            // Process stdout response from the server.
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                results.push(format!(""));
                results.push(format!("Server Response:"));

                // Try to parse and pretty-print JSON response.
                match json_from_str::<Value>(&stdout) {
                    Ok(json) => match to_string_pretty(&json) {
                        Ok(pretty) => results.push(format!("{}", pretty)), // Pretty-printed JSON.
                        Err(_) => results.push(format!("{}", stdout)), // Raw response if pretty-printing fails.
                    },
                    Err(_) => results.push(format!("{}", stdout)), // Raw response if not valid JSON.
                }
            }

            // Add any error information from stderr if request failed.
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                results.push(format!(""));
                results.push(format!("Error Details:"));
                results.push(format!("{}", stderr));
            }
        }
        Err(e) => {
            // Handle case where the command execution itself failed.
            results.push(format!(""));
            results.push(format!("Failed to execute test: {}", e));
        }
    }
}

/*
 * Calculate wait time for test completion
 * Determines how long to wait for a test to complete based on its
 * configured duration, adding a small buffer to ensure the test
 * has fully finished before checking results.
 */
fn calculate_wait_time(duration: &str) -> u64 {
    match duration.parse::<u64>() {
        Ok(d) => d + 2, // Add a 2-second buffer to the specified duration
        Err(_) => 10,   // Default to 10 seconds if parsing fails
    }
}

/*
 * Check test status after completion
 * This asynchronous function queries the API for the final status
 * of a test after it should have completed. It:
 * - Makes a request to the status endpoint
 * - Processes and formats the status response
 * - Extracts and displays relevant metrics
 */
async fn check_test_status(
    results: &mut Vec<String>,  // Vector to append status information to.
    test: &TestType,            // Reference to the TestType that was executed.
    server_url: &str,           // Base URL of the test server.
    test_id: &str,              // Unique identifier for the test.
) {
    // Construct command to query test status.
    let status_command = format!("curl -X GET {}/status/{}", server_url, test_id);
    results.push(format!("Checking test status..."));

    // Execute the status query command.
    let status_output = ProcessCommand::new("sh")
        .arg("-c")
        .arg(&status_command)
        .output();

    match status_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.trim().is_empty() {
                    // Process non-empty status response.
                    results.push(format!(""));
                    results.push(format!("Final Test Status:"));

                    // Try to parse response as JSON.
                    match json_from_str::<Value>(&stdout) {
                        Ok(json) => {
                            // Extract overall test status.
                            if let Some(status) = json.get("status") {
                                if let Some(status_str) = status.as_str() {
                                    results.push(format!("  • Status: {}", status_str));
                                }
                            }

                            // Process test-specific metrics.
                            process_test_metrics(results, test, &json);
                        }
                        // If not valid JSON, include raw response.
                        Err(_) => results.push(format!("{}", stdout)),
                    }
                } else {
                    // Handle empty response.
                    results.push(format!("No status information available."));
                }
            } else {
                // Handle unsuccessful HTTP status.
                let stderr = String::from_utf8_lossy(&output.stderr);
                results.push(format!("Failed to get status: {}", stderr));
            }
        }
        Err(e) => {
            // Handle command execution failure.
            results.push(format!("Error checking test status: {}", e));
        }
    }
}

/**
 * Process test metrics from status response
 * Extracts and formats test-specific metrics from the status response JSON.
 * Different metrics are relevant for each test type:
 * - CPU: Usage percentages and thread counts
 * - Memory: Allocation sizes and system memory usage
 * - Disk: Read/write speeds and total I/O
 */
fn process_test_metrics(results: &mut Vec<String>, test: &TestType, json: &Value) {
    // Check if metrics section exists in the response.
    if let Some(metrics) = json.get("metrics") {
        results.push(format!(""));
        results.push(format!("Test Metrics:"));

        match test {
            TestType::Cpu => {
                // === CPU TEST METRICS ===
                // CPU usage percentage across the system.
                if let Some(cpu_usage) = metrics.get("cpu_usage") {
                    results.push(format!("  • CPU Usage: {}", cpu_usage));
                }
                // Total thread count used during test.
                if let Some(thread_count) = metrics.get("thread_count") {
                    results.push(format!("  • Thread Count: {}", thread_count));
                }
            }
            TestType::Memory => {
                // === MEMORY TEST METRICS ===
                // Total memory allocated by the test.
                if let Some(allocated) = metrics.get("allocated_mb") {
                    results.push(format!("  • Allocated Memory: {} MB", allocated));
                }
                // System-wide memory statistics reported by the test.
                if let Some(total) = metrics.get("total_memory_mb") {
                    results.push(format!("  • Total System Memory: {} MB", total));
                }
                if let Some(used) = metrics.get("used_memory_mb") {
                    results.push(format!("  • Used System Memory: {} MB", used));
                }

                // Get post-test memory information for comparison with pre-test state.
                let final_memory = get_memory_info();
                results.push(format!("  • System Memory Information (Post-Test):"));
                if let Some((total, used)) = final_memory {
                    results.push(format!("    - Total Memory: {} MB", total));
                    results.push(format!("    - Used Memory: {} MB", used));
                    results.push(format!("    - Free Memory: {} MB", total - used));
                } else {
                    results.push(format!("    - Memory information not available"));
                }
            }
            TestType::Disk => {
                // === DISK TEST METRICS ===
                // Write performance metrics.
                if let Some(write_speed) = metrics.get("write_speed_mb_s") {
                    results.push(format!("  • Write Speed: {} MB/s", write_speed));
                }
                // Read performance metrics.
                if let Some(read_speed) = metrics.get("read_speed_mb_s") {
                    results.push(format!("  • Read Speed: {} MB/s", read_speed));
                }
                // Total data transferred during test.
                if let Some(total) = metrics.get("total_io_mb") {
                    results.push(format!("  • Total I/O: {} MB", total));
                }
            }
        }
    }
}
/*
 * Add summary section to results.
 * This function creates a comprehensive summary section at the end of the test report,
 * including:
 * - Batch identifier for tracking.
 * - Count of executed tests.
 * - List of test types that were executed.
 * - Timestamp marking completion of all tests.
 */
fn add_summary_section(results: &mut Vec<String>, batch_id: &str, selected_tests: &[TestType]) {
    // Add section header.
    results.push(format!("===================================="));
    results.push(format!("TEST SUMMARY"));
    results.push(format!("===================================="));
    // Add batch tracking information.
    results.push(format!("Batch ID: {}", batch_id));
    // Add test count and list of executed test types.
    results.push(format!("Tests Executed: {}", selected_tests.len()));
    results.push(format!(
        "Tests: {}",
        selected_tests
            .iter()
            .map(|t| match t {
                TestType::Cpu => "CPU",
                TestType::Memory => "Memory",
                TestType::Disk => "Disk",
            })
            .collect::<Vec<_>>()
            .join(", ") // Join test names with commas.
    ));
    // Add timestamp for when all tests completed.
    results.push(format!(
        "Completed at: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
}

/*
 * This function initializes and launches the Iced application with default settings.
 * The application follows Iced's ELM architecture:
 * - Model: The GuiApp struct containing application state
 * - View: The view() method rendering the UI
 * - Update: The update() method handling events
 */
pub fn run() -> iced::Result {
    GuiApp::run(Settings::default())
}

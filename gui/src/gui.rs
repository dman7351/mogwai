/**
 * Mogwai Test GUI - Performance Testing Utility
 * CS 488 Senior Project
 *
 * This application provides a graphical interface for running CPU, memory, and disk
 * stress tests across different environments.
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

// ===== ENVIRONMENT CONFIGURATION =====
/**
 * Defines available environments for running tests
 * - Local: Local development environment (http://localhost:8080)
 * - Kubernetes: Kubernetes cluster environment (http://localhost:8081)
 * - Custom: Custom URL environment specified by the user
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Local,      // Local development environment
    Kubernetes, // Kubernetes cluster environment
    Custom,     // Custom URL environment
}
// Default environment is Local
impl Default for Environment {
    fn default() -> Self {
        Environment::Local
    }
}
// Display for environment types
impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Kubernetes => write!(f, "Kubernetes"),
            Self::Custom => write!(f, "Custom URL"),
        }
    }
}

// ===== APPLICATION MESSAGES =====
/**
 * Message types for handling user interactions and async operations
 * This enum defines all possible events that can occur in the application
 */
#[derive(Debug, Clone)]
pub enum Message {
    ToggleTest(TestType, bool), // Message to toggle the selection of a test type (test type, is checked)
    RunPressed,                 // Message when the "Run Tests" button is pressed
    ListTasksPressed,           // Message when the "List Tasks" button is pressed
    ServerUrlChanged(String),   // Message when the server URL input field changes (new URL value)
    DurationChanged(String), // Message when the test duration input field changes (new duration value)
    IntensityChanged(String), // Message when the test intensity input field changes (new intensity value)
    SizeChanged(String),      // Message when the test size input field changes (new size value)
    LoadChanged(String), // Message when the CPU load percentage input field changes (new load value)
    ForkToggled(bool),   // Message when the "Fork Test" toggle is changed (new toggle state)
    ToggleAdvanced,      // Message to toggle the visibility of advanced settings
    TestComplete(String), // Message received when a test execution completes (test results as a string)
    TasksListed(String),  // Message received with the list of running tasks (as a string)
    EnvironmentSelected(Environment), // Message when a different environment is selected from the dropdown (new environment)
    #[allow(dead_code)]
    LogsReceived(String), // Message received containing logs from the test execution (as a string, currently not fully used in UI)
    NodeStatusReceived(String), // Message received with the status of the nodes involved in the test (as a string)
    SaveResultsPressed,         // Message when the "Save Results" button is pressed
    ResultsSaved(Result<(), String>), // Message indicating the result of the save operation (Ok for success, Err with error message)
}
// ===== TEST TYPES =====
///Types of stress tests available in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestType {
    Cpu,    //CPU stress test
    Memory, //memory stress test
    Disk,   //disk stress test
}

// ===== MAIN APPLICATION STRUCT =====
/**
 * Main application state container
 * Holds all current configuration, status, and result information
 */
pub struct GuiApp {
    // Test selection and parameters
    selected_tests: Vec<TestType>, // Vector to store the currently selected test types
    server_url: String,            // The URL of the server to send test requests to
    environment: Environment,      // The currently selected environment
    duration: String,              // The duration of the tests, as a string from user input
    intensity: String, // The intensity of the tests (e.g., number of threads), as a string
    size: String,      // The size parameter for memory and disk tests (in MB), as a string
    load: String,      // The CPU load percentage for the CPU test, as a string
    fork: bool,        // Flag indicating if the CPU test should fork separate processes

    // State tracking
    status_message: Option<String>, // Message to display status updates and results to the user
    node_status: Option<String>,    // Status information received from the test nodes
    test_results: Option<String>,   // The raw results of the completed tests
    show_advanced: bool,            // Flag to control the visibility of advanced settings
    running_tests: bool,            // Flag to indicate if tests are currently running
    last_test_id: Option<String>, // The ID of the last run test batch, used for fetching node status
}

// === APPLICATION IMPLEMENTATION ===
impl Application for GuiApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    /**
     * Initialize the application with default settings
     */
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            GuiApp {
                selected_tests: vec![],
                server_url: String::from("http://localhost:8080"),
                environment: Environment::Local,
                duration: String::from("10"),
                intensity: String::from("4"),
                size: String::from("256"),
                load: String::from("70.0"),
                fork: false,
                status_message: None,
                node_status: None,
                show_advanced: false,
                running_tests: false,
                test_results: None,
                last_test_id: None,
            },
            Command::none(),
        )
    }
    //Set application window title
    fn title(&self) -> String {
        "Mogwai Test GUI".into()
    }

    /// Handle all application events and update state accordingly
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {

            // === INPUT FIELD CHANGES ===
            // Toggle a test type selection (checked/unchecked)
            Message::ToggleTest(test, checked) => {
                if checked && !self.selected_tests.contains(&test) {
                    self.selected_tests.push(test);
                } else {
                    self.selected_tests.retain(|&t| t != test);
                }
            }
            Message::ServerUrlChanged(url) => self.server_url = url, // Update the server URL in the application state
            Message::DurationChanged(duration) => self.duration = duration, // Update the test duration in the application state
            Message::IntensityChanged(intensity) => self.intensity = intensity, // Update the test intensity in the application state
            Message::SizeChanged(size) => self.size = size, // Update the test size in the application state
            Message::LoadChanged(load) => self.load = load, // Update the CPU load percentage in the application state
            Message::ForkToggled(fork) => self.fork = fork, // Update the fork option in the application state
            Message::ToggleAdvanced => self.show_advanced = !self.show_advanced, // Toggle the visibility of advanced settings
            Message::EnvironmentSelected(env) => {
                self.environment = env; // Update the selected environment in the application state
                self.server_url = match env {
                    Environment::Local => "http://localhost:8080".to_string(), // Set default URL for Local environment
                    Environment::Kubernetes => "http://localhost:8081".to_string(), // Set default URL for Kubernetes environment
                    Environment::Custom => self.server_url.clone(), // Keep the existing custom URL
                };
            }

            // === TEST EXECUTION & RESULTS ===
            // Handle test completion
            Message::TestComplete(results) => {
                self.running_tests = false; // Reset the running tests flag
                self.status_message = Some(results.clone()); // Update the status message with the test results
                self.test_results = Some(results); // Store the test results in the application state

                // Fetch node status as needed
                if let Some(test_id) = &self.last_test_id {
                    return fetch_node_status(self.server_url.clone(), test_id.clone());
                }
            }
            Message::NodeStatusReceived(status) => {
                self.node_status = Some(status); // Update the displayed node status
            }
            Message::LogsReceived(logs) => {
                if let Some(existing) = &self.node_status {
                    self.node_status = Some(format!("{}\n\nLogs:\n{}", existing, logs));
                } else {
                    self.node_status = Some(format!("Logs:\n{}", logs));
                }
            }

            // Actions
            Message::SaveResultsPressed => {
                if let Some(results) = &self.test_results {
                    return save_results(results.clone());
                } // Initiate the process of saving the test results to a file
            }
            Message::ResultsSaved(result) => match result {
                Ok(_) => {
                    self.status_message = Some(format!(
                        "{}\n\nResults successfully saved to results directory.",
                        self.status_message.clone().unwrap_or_default()
                    ));// Update the status message on successful saving of results
                }
                Err(e) => {
                    self.status_message = Some(format!(
                        "{}\n\nFailed to save results: {}",
                        self.status_message.clone().unwrap_or_default(),
                        e
                    )); // Update status on save failure
                }
            },
            Message::TasksListed(results) => {
                self.status_message = Some(results);
            } // Update status with the list of tasks

            Message::ListTasksPressed => {
                self.status_message = Some("Fetching running tasks...".to_string());
                return list_tasks(self.server_url.clone());
            }
            Message::RunPressed => {
                // Validation
                if self.selected_tests.is_empty() {
                    self.status_message = Some("No tests selected.".to_string());
                    return Command::none();
                }

                if self.fork && !self.selected_tests.contains(&TestType::Cpu) {
                    self.status_message =
                        Some("Fork option requires CPU test to be selected.".to_string());
                    return Command::none();
                }

                // Update state
                self.running_tests = true;
                self.status_message = Some("Running tests...".to_string());

                // Generate batch ID
                let batch_id = Uuid::new_v4().to_string();
                self.last_test_id = Some(batch_id.clone());

                // Run tests
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
                    Message::TestComplete,  // Send Message::TestComplete when the async operation finishes
                );
            }
        }
        Command::none() // Default case: no command to execute
    }

    /// Render application UI
    fn view(&self) -> Element<'_, Self::Message> {
        // Header
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

        let separator = Rule::horizontal(1);

        // Advanced toggle
        let advanced_toggle = Row::new()
            .push(Text::new("Advanced Settings").size(16))
            .push(Space::with_width(Length::Fill))
            .push(
                toggler(None, self.show_advanced, |_| Message::ToggleAdvanced)
                    .width(Length::Fixed(40.0)),
            )
            .width(Length::Fill)
            .align_items(Alignment::Center);

        // Advanced settings section (collapsible)
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
            Column::new()
        };

        // Test selection checkboxes
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

        // Parameter inputs
        let params_title = Text::new("Test Parameters:").size(18);

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

        // CPU options
        let fork_section = Column::new()
            .push(Text::new("CPU Test Options:").size(18))
            .push(
                Container::new(Checkbox::new("Fork Test", self.fork, Message::ForkToggled))
                    .padding(5),
            )
            .spacing(5)
            .width(Length::Fill);

        // Parameter help text
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

        // Action buttons
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

        let list_tasks_button = Button::new(
            Text::new("LIST TASKS")
                .size(16)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .on_press(Message::ListTasksPressed)
        .padding([8, 20])
        .style(iced::theme::Button::Secondary)
        .width(Length::Fill);

        let save_button = Button::new(
            Text::new("SAVE RESULTS")
                .size(16)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .on_press(Message::SaveResultsPressed)
        .padding([8, 20])
        .style(iced::theme::Button::Secondary)
        .width(Length::Fill);

        // Button layouts
        let primary_button_row = Row::new()
            .push(Container::new(run_button).width(Length::FillPortion(2)))
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(Container::new(list_tasks_button).width(Length::FillPortion(1)))
            .spacing(10)
            .width(Length::Fixed(450.0));

        let secondary_button_row = Row::new()
            .push(Container::new(save_button).width(Length::Fill))
            .spacing(10)
            .width(Length::Fixed(450.0));

        // Results display
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

        // Main layout
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

        Container::new(Scrollable::new(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .padding(30)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}

// === HELPER FUNCTIONS ===
/// Fetch node status for a test
fn fetch_node_status(server_url: String, test_id: String) -> Command<Message> {
    Command::perform(
        async move {
            // Sleep to give the test time to propagate to nodes
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            // Fetch node status from API
            let endpoint = format!("{}/nodes/{}", server_url, test_id);
            println!("Fetching node status from: {}", endpoint);

            let command = format!("curl -X GET {}", endpoint);
            let output = ProcessCommand::new("sh").arg("-c").arg(&command).output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);

                        if stdout.trim().is_empty() {
                            "No node status available.".to_string()
                        } else {
                            // Try to parse as JSON and format it nicely
                            match json_from_str::<Value>(&stdout) {
                                Ok(json) => {
                                    // Check if it's a "Not Found" error
                                    if let Some(detail) = json.get("detail") {
                                        if detail.as_str() == Some("Not Found") {
                                            format!("Node Status for Test {}:\n\nNo detailed node status available.", test_id)
                                        } else {
                                            // Format the node status nicely
                                            format!(
                                                "Node Status for Test {}:\n\n{}",
                                                test_id,
                                                format_node_status(&stdout)
                                            )
                                        }
                                    } else {
                                        // Format the node status nicely
                                        format!(
                                            "Node Status for Test {}:\n\n{}",
                                            test_id,
                                            format_node_status(&stdout)
                                        )
                                    }
                                }
                                Err(_) => format!("Node Status for Test {}:\n{}", test_id, stdout),
                            }
                        }
                    } else {
                        "Failed to fetch node status.".to_string()
                    }
                }
                Err(_) => "Error connecting to server for node status.".to_string(),
            }
        },
        Message::NodeStatusReceived,
    )
}

/// Save test results to a file
fn save_results(results: String) -> Command<Message> {
    Command::perform(
        async move {
            // Create results directory if it doesn't exist
            let results_dir = Path::new("results");
            if !results_dir.exists() {
                if let Err(e) = fs::create_dir_all(results_dir) {
                    return Err(format!("Failed to create results directory: {}", e));
                }
            }

            // Generate filename with timestamp
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let filename = format!("mogwai_results_{}.txt", timestamp);
            let path = results_dir.join(filename);

            // Write results to file
            match File::create(&path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(results.as_bytes()) {
                        return Err(format!("Failed to write to file: {}", e));
                    }
                    Ok(())
                }
                Err(e) => Err(format!("Failed to create file: {}", e)),
            }
        },
        Message::ResultsSaved,
    )
}

/// List running tasks
fn list_tasks(server_url: String) -> Command<Message> {
    Command::perform(
        async move {
            let endpoint = format!("{}/tasks", server_url);
            println!("Fetching tasks from: {}", endpoint);

            let command = format!("curl -X GET {}", endpoint);
            let output = ProcessCommand::new("sh").arg("-c").arg(&command).output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.trim().is_empty() {
                            "No running tasks found.".to_string()
                        } else {
                            parse_tasks_response(&stdout)
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        format!("Failed to get tasks: {}", stderr)
                    }
                }
                Err(e) => format!("Error fetching tasks: {}", e),
            }
        },
        Message::TasksListed,
    )
}

/// Format node status JSON into readable text
fn format_node_status(json_str: &str) -> String {
    match json_from_str::<Value>(json_str) {
        Ok(json) => {
            if let Some(detail) = json.get("detail") {
                if detail.as_str() == Some("Not Found") {
                    return "Node status data not found for this test ID.".to_string();
                }
            }

            let mut result = String::new();

            // Extract basic test info
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

            // Add sections for node info and metrics (simplified version)
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

            if result.is_empty() {
                return format!("Raw node status data:\n{}", json_str);
            }

            result
        }
        Err(_) => format!("Raw response (not valid JSON):\n{}", json_str),
    }
}

/// Helper to format a JSON section in node status output
fn format_json_section(result: &mut String, section: Option<&Value>, header: &str) {
    if let Some(section_value) = section {
        if let Some(obj) = section_value.as_object() {
            if !obj.is_empty() {
                result.push_str(header);
                for (key, value) in obj {
                    if value.is_object() {
                        result.push_str(&format!("{}:\n", key));
                        if let Some(inner_obj) = value.as_object() {
                            for (inner_key, inner_value) in inner_obj {
                                let display_value = format_json_value(inner_value);
                                result.push_str(&format!("  {}: {}\n", inner_key, display_value));
                            }
                        }
                    } else {
                        let display_value = format_json_value(value);
                        result.push_str(&format!("{}: {}\n", key, display_value));
                    }
                }
            }
        }
    }
}

/// Format a JSON value as a string
fn format_json_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => value.to_string(),
    }
}

/// Parse and format task list response
fn parse_tasks_response(stdout: &str) -> String {
    match json_from_str::<Value>(stdout) {
        Ok(json) => {
            // Check if it's a "Not Found" error
            if let Some(detail) = json.get("detail") {
                if detail.as_str() == Some("Not Found") {
                    return "=== RUNNING TASKS ===\n\nNo task list available.".to_string();
                }
            } else if stdout.contains("[") && stdout.contains("]") {
                // It's probably a JSON array
                let mut result = String::from("=== RUNNING TASKS ===\n\n");

                // Try to parse as an array
                if let Some(tasks) = json.as_array() {
                    if tasks.is_empty() {
                        result.push_str("No running tasks found.\n");
                    } else {
                        result.push_str(&format!("Found {} running tasks:\n\n", tasks.len()));

                        for (i, task) in tasks.iter().enumerate() {
                            format_task_item(&mut result, i, task);
                        }
                    }
                    return result;
                }
            }

            // Default: show raw JSON
            format!("=== RUNNING TASKS ===\n\n{}", stdout)
        }
        Err(_) => format!("=== RUNNING TASKS ===\n\n{}", stdout),
    }
}

/// Format a single task item in the task list
fn format_task_item(result: &mut String, index: usize, task: &Value) {
    if let Some(task_obj) = task.as_object() {
        result.push_str(&format!("Task #{}: \n", index + 1));
        for (key, value) in task_obj {
            let display_value = format_json_value(value);
            result.push_str(&format!("  • {}: {}\n", key, display_value));
        }
        result.push_str("\n");
    } else if let Some(task_str) = task.as_str() {
        result.push_str(&format!("Task #{}: {}\n", index + 1, task_str));
    } else {
        result.push_str(&format!("Task #{}: {}\n", index + 1, task.to_string()));
    }
}

/**
* Get system information for test reports
*/
fn get_system_info() -> String {
    let mut info = Vec::new();

    // Try to get OS information
    #[cfg(target_os = "linux")]
    {
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

    // Get CPU and memory information
    #[cfg(target_os = "linux")]
    {
        // CPU info
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

        // CPU cores
        if let Ok(output) = ProcessCommand::new("sh").arg("-c").arg("nproc").output() {
            let cpu_count = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !cpu_count.is_empty() {
                info.push(format!("CPU Cores: {}", cpu_count));
            }
        }

        // Memory
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

    // If couldn't get any information, provide a fallback
    if info.is_empty() {
        return "System information not available.".to_string();
    }

    // Join all the information lines
    info.join("\n")
}

/**
* Get memory information from the system
*/
fn get_memory_info() -> Option<(u64, u64)> {
    #[cfg(target_os = "linux")]
    {
        // Try to get total memory
        if let Ok(output) = ProcessCommand::new("sh")
            .arg("-c")
            .arg("free -m | grep Mem | awk '{print $2}'")
            .output()
        {
            let total_mem = String::from_utf8_lossy(&output.stdout).trim().to_string();

            // Try to get used memory
            if let Ok(output) = ProcessCommand::new("sh")
                .arg("-c")
                .arg("free -m | grep Mem | awk '{print $3}'")
                .output()
            {
                let used_mem = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
    
    None
}

/// Execute tests with full metrics and reporting
async fn execute_tests(
    selected_tests: Vec<TestType>,
    server_url: String,
    batch_id: String,
    duration: String,
    intensity: String,
    size: String,
    load: String,
    fork: bool,
) -> String {
    let mut results = Vec::new();

    // Add report header
    add_report_header(&mut results, &batch_id);

    // Add system information
    results.push(format!("SYSTEM INFORMATION"));
    results.push(format!("------------------------------------"));
    results.push(get_system_info());
    results.push(format!(""));

    // Process each selected test
    for test in &selected_tests {
        // Add test header
        let test_name = get_test_name(test);
        add_test_header(&mut results, test_name);

        // Generate test ID and prepare payload
        let test_id = Uuid::new_v4().to_string();
        let (endpoint, payload) = prepare_test_payload(
            test, &test_id, &batch_id, &duration, &intensity, &size, &load, fork,
        );

        // Add request details
        add_request_details(&mut results, &server_url, endpoint, &test_id);

        // Add test parameters based on test type
        add_test_parameters(
            &mut results,
            test,
            &duration,
            &intensity,
            &size,
            &load,
            fork,
        );

        // Add payload for reference
        results.push(format!(""));
        results.push(format!("JSON Payload:"));
        results.push(format!("{}", payload));

        // Execute the test
        let command = format!(
            "curl -X POST {}/{} -H \"Content-Type:application/json\" -d '{}'",
            server_url, endpoint, payload
        );

        // Process response
        let output = ProcessCommand::new("sh").arg("-c").arg(&command).output();
        process_test_response(&mut results, output);

        // Wait for test completion
        results.push(format!(""));
        results.push(format!(
            "Test {} started, waiting for completion...",
            test_name
        ));

        let wait_time = calculate_wait_time(&duration);
        tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;

        // Check for test results via status endpoint
        check_test_status(&mut results, test, &server_url, &test_id).await;

        // Add test completion marker
        results.push(format!(""));
        results.push(format!("Test {} completed.", test_name));
        results.push(format!(""));
    }

    // Add summary section
    add_summary_section(&mut results, &batch_id, &selected_tests);

    // Return the complete results
    results.join("\n")
}

/// Add report header to results
fn add_report_header(results: &mut Vec<String>, batch_id: &str) {
    results.push(format!("===================================="));
    results.push(format!("MOGWAI PERFORMANCE TEST REPORT"));
    results.push(format!("===================================="));
    results.push(format!(
        "Date/Time: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    results.push(format!("Batch ID: {}", batch_id));
    results.push(format!(""));
}

/// Get user-friendly test name
fn get_test_name(test: &TestType) -> &'static str {
    match test {
        TestType::Cpu => "CPU",
        TestType::Memory => "Memory",
        TestType::Disk => "Disk",
    }
}

/// Add test header to results
fn add_test_header(results: &mut Vec<String>, test_name: &str) {
    results.push(format!("===================================="));
    results.push(format!("RUNNING {} TEST", test_name));
    results.push(format!("===================================="));
}

/// Prepare payload for test
fn prepare_test_payload(
    test: &TestType,
    test_id: &str,
    batch_id: &str,
    duration: &str,
    intensity: &str,
    size: &str,
    load: &str,
    fork: bool,
) -> (&'static str, String) {
    let endpoint = match test {
        TestType::Cpu => "cpu-stress",
        TestType::Memory => "mem-stress",
        TestType::Disk => "disk-stress",
    };

    let payload = match test {
        TestType::Cpu => {
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
            format!(
                r#"{{"id": "{}", "batch_id": "{}", "name": "GUI Test", "intensity": {}, "duration": {}, "size": {}}}"#,
                test_id, batch_id, intensity, duration, size
            )
        }
    };

    (endpoint, payload)
}

/// Add request details to results
fn add_request_details(results: &mut Vec<String>, server_url: &str, endpoint: &str, test_id: &str) {
    results.push(format!("Request Details:"));
    results.push(format!("  Endpoint: {}/{}", server_url, endpoint));
    results.push(format!("  Test ID: {}", test_id));
}

/// Add test parameters to results
fn add_test_parameters(
    results: &mut Vec<String>,
    test: &TestType,
    duration: &str,
    intensity: &str,
    size: &str,
    load: &str,
    fork: bool,
) {
    results.push(format!("Test Parameters:"));

    match test {
        TestType::Cpu => {
            results.push(format!("  • Threads: {}", intensity));
            results.push(format!("  • Duration: {} seconds", duration));
            results.push(format!("  • Target CPU Load: {}%", load));

            // Add fork-specific information
            if fork {
                results.push(format!("  • Fork Mode: Enabled (using separate processes)"));
                results.push(format!("  • Process Count: {} processes", intensity));

                // Add explanation of fork mode
                results.push(format!("  • Fork Mode Details:"));
                results.push(format!("    - Each process runs independently"));
                results.push(format!("    - Parent process monitors child processes"));
                results.push(format!(
                    "    - System resources allocated separately for each process"
                ));
            } else {
                results.push(format!("  • Fork Mode: Disabled (using threads)"));

                // Add CPU mode explanation
                if let Ok(load_val) = load.parse::<f64>() {
                    if load_val < 100.0 {
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
                        results.push(format!(
                            "  • CPU Cycle Details: Running at full capacity (100% busy loop)"
                        ));
                    }
                }
            }
        }
        TestType::Memory => {
            results.push(format!("  • Threads: {}", intensity));
            results.push(format!("  • Duration: {} seconds", duration));
            results.push(format!("  • Size per Thread: {} MB", size));

            // Calculate and add total memory allocation
            if let (Ok(threads), Ok(size_mb)) = (intensity.parse::<usize>(), size.parse::<usize>())
            {
                let total_mb = threads * size_mb;
                results.push(format!("  • Total Memory Allocation: {} MB", total_mb));

                // Add memory test details
                results.push(format!("  • Memory Test Details:"));
                results.push(format!("    - Each thread allocates blocks of memory"));
                results.push(format!(
                    "    - Memory is actively used to prevent optimization"
                ));
                results.push(format!("    - 4KB page size access pattern"));
            }

            // Get initial memory information
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
            results.push(format!("  • Threads: {}", intensity));
            results.push(format!("  • Duration: {} seconds", duration));
            results.push(format!("  • File Size: {} MB", size));

            // Calculate and add total disk usage
            if let (Ok(threads), Ok(size_mb)) = (intensity.parse::<usize>(), size.parse::<usize>())
            {
                let total_mb = threads * size_mb;
                results.push(format!("  • Total Disk Usage: {} MB", total_mb));

                // Add disk test details
                results.push(format!("  • Disk Test Details:"));
                results.push(format!("    - Each thread creates a separate file"));
                results.push(format!("    - Alternating write and read phases"));
                results.push(format!("    - Files are cleaned up after test"));
                results.push(format!("    - Sequential I/O pattern"));
            }
        }
    }
}

/// Process test response
fn process_test_response(
    results: &mut Vec<String>,
    output: Result<std::process::Output, std::io::Error>,
) {
    match output {
        Ok(output) => {
            let status_str = if output.status.success() {
                "SUCCESS"
            } else {
                "FAILED"
            };
            results.push(format!(""));
            results.push(format!("Execution Status: {}", status_str));

            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                results.push(format!(""));
                results.push(format!("Server Response:"));

                // Try to parse as JSON for better formatting
                match json_from_str::<Value>(&stdout) {
                    Ok(json) => match to_string_pretty(&json) {
                        Ok(pretty) => results.push(format!("{}", pretty)),
                        Err(_) => results.push(format!("{}", stdout)),
                    },
                    Err(_) => results.push(format!("{}", stdout)),
                }
            }

            // Add any error information
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                results.push(format!(""));
                results.push(format!("Error Details:"));
                results.push(format!("{}", stderr));
            }
        }
        Err(e) => {
            results.push(format!(""));
            results.push(format!("Failed to execute test: {}", e));
        }
    }
}

/// Calculate wait time for test completion
fn calculate_wait_time(duration: &str) -> u64 {
    match duration.parse::<u64>() {
        Ok(d) => d + 2, // Add a small buffer
        Err(_) => 10,   // Default to 10 seconds if parsing fails
    }
}

/// Check test status after completion
async fn check_test_status(
    results: &mut Vec<String>,
    test: &TestType,
    server_url: &str,
    test_id: &str,
) {
    let status_command = format!("curl -X GET {}/status/{}", server_url, test_id);
    results.push(format!("Checking test status..."));

    let status_output = ProcessCommand::new("sh")
        .arg("-c")
        .arg(&status_command)
        .output();

    match status_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.trim().is_empty() {
                    results.push(format!(""));
                    results.push(format!("Final Test Status:"));

                    match json_from_str::<Value>(&stdout) {
                        Ok(json) => {
                            // Get test status
                            if let Some(status) = json.get("status") {
                                if let Some(status_str) = status.as_str() {
                                    results.push(format!("  • Status: {}", status_str));
                                }
                            }

                            // Extract metrics
                            process_test_metrics(results, test, &json);
                        }
                        Err(_) => results.push(format!("{}", stdout)),
                    }
                } else {
                    results.push(format!("No status information available."));
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                results.push(format!("Failed to get status: {}", stderr));
            }
        }
        Err(e) => {
            results.push(format!("Error checking test status: {}", e));
        }
    }
}

/// Process test metrics from status response
fn process_test_metrics(results: &mut Vec<String>, test: &TestType, json: &Value) {
    if let Some(metrics) = json.get("metrics") {
        results.push(format!(""));
        results.push(format!("Test Metrics:"));

        match test {
            TestType::Cpu => {
                if let Some(cpu_usage) = metrics.get("cpu_usage") {
                    results.push(format!("  • CPU Usage: {}", cpu_usage));
                }
                if let Some(thread_count) = metrics.get("thread_count") {
                    results.push(format!("  • Thread Count: {}", thread_count));
                }
            }
            TestType::Memory => {
                if let Some(allocated) = metrics.get("allocated_mb") {
                    results.push(format!("  • Allocated Memory: {} MB", allocated));
                }
                if let Some(total) = metrics.get("total_memory_mb") {
                    results.push(format!("  • Total System Memory: {} MB", total));
                }
                if let Some(used) = metrics.get("used_memory_mb") {
                    results.push(format!("  • Used System Memory: {} MB", used));
                }

                // Get post-test memory information
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
                if let Some(write_speed) = metrics.get("write_speed_mb_s") {
                    results.push(format!("  • Write Speed: {} MB/s", write_speed));
                }
                if let Some(read_speed) = metrics.get("read_speed_mb_s") {
                    results.push(format!("  • Read Speed: {} MB/s", read_speed));
                }
                if let Some(total) = metrics.get("total_io_mb") {
                    results.push(format!("  • Total I/O: {} MB", total));
                }
            }
        }
    }
}

/// Add summary section to results
fn add_summary_section(results: &mut Vec<String>, batch_id: &str, selected_tests: &[TestType]) {
    results.push(format!("===================================="));
    results.push(format!("TEST SUMMARY"));
    results.push(format!("===================================="));
    results.push(format!("Batch ID: {}", batch_id));
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
            .join(", ")
    ));
    results.push(format!(
        "Completed at: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
}

/// Entry point to run the application
pub fn run() -> iced::Result {
    GuiApp::run(Settings::default())
}

use iced::widget::{Button, Checkbox, Column, Container, Text, TextInput, Row, Space, Scrollable};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};
use iced::widget::toggler;
use std::process::Command as ProcessCommand;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTest(TestType, bool),
    RunPressed,
    ServerUrlChanged(String),
    DurationChanged(String),
    IntensityChanged(String),
    SizeChanged(String),
    LoadChanged(String),
    ForkToggled(bool),
    ToggleAdvanced,
    TestComplete(String),
    NodeChanged(String)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestType {
    Cpu,
    Memory,
    Disk,
}

pub struct GuiApp {
    // Selected tests
    selected_tests: Vec<TestType>,
   
    // Server configuration
    server_url: String,
   
    // Test parameters
    duration: String,
    intensity: String,
    size: String,
    load: String,
    fork: bool,
    node: String,
   
    // Status message to display results
    status_message: Option<String>,
    
    // New fields
    show_advanced: bool,
    running_tests: bool,
}

impl Application for GuiApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            GuiApp {
                selected_tests: vec![],
                server_url: String::from("http://localhost:8080"),
                duration: String::from("10"),
                intensity: String::from("4"),
                size: String::from("256"),
                load: String::from("100.0"),
                fork: false,
                node: String::new(),
                status_message: None,
                show_advanced: false,
                running_tests: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Mogwai Test GUI".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ToggleTest(test, checked) => {
                if checked {
                    if !self.selected_tests.contains(&test) {
                        self.selected_tests.push(test);
                    }
                } else {
                    self.selected_tests.retain(|&t| t != test);
                }
                Command::none()
            }
            Message::ServerUrlChanged(url) => {
                self.server_url = url;
                Command::none()
            }
            Message::DurationChanged(duration) => {
                self.duration = duration;
                Command::none()
            }
            Message::IntensityChanged(intensity) => {
                self.intensity = intensity;
                Command::none()
            }
            Message::SizeChanged(size) => {
                self.size = size;
                Command::none()
            }
            Message::LoadChanged(load) => {
                self.load = load;
                Command::none()
            }
            Message::ForkToggled(fork) => {
                self.fork = fork;
                Command::none()
            }
            Message::NodeChanged(n) => {
                self.node = n;
                Command::none()
            }
            Message::ToggleAdvanced => {
                self.show_advanced = !self.show_advanced;
                Command::none()
            }
            Message::TestComplete(results) => {
                self.running_tests = false;
                self.status_message = Some(results); // Update the status message with the results
                Command::none()
            }
            Message::RunPressed => {
                println!("Run button pressed!");
                if self.selected_tests.is_empty() {
                    self.status_message = Some("No tests selected. Please select at least one test.".to_string());
                    return Command::none();
                }
                
                // Update running state
                self.running_tests = true;
                self.status_message = Some("Running tests...".to_string());
                
                // Start running tests asynchronously
                let selected_tests = self.selected_tests.clone();
                let server_url = self.server_url.clone();
                let duration = self.duration.clone();
                let intensity = self.intensity.clone();
                let size = self.size.clone();
                let load = self.load.clone();
                let fork = self.fork;
                let node = self.node.clone();
                
                return Command::perform(
                    async move {
                        let mut results = Vec::new();
                        
                        for test in &selected_tests {
                            let endpoint = match test {
                                TestType::Cpu => "cpu-stress",
                                TestType::Memory => "mem-stress",
                                TestType::Disk => "disk-stress",
                            };
                        
                            // Build the curl command with appropriate parameters
                            let payload = match test {
                                TestType::Cpu => {
                                    format!(
                                        r#"{{"intensity": {}, "duration": {}, "load": {}, "fork": {}, "node": "{}"}}"#,
                                        intensity, duration, load, fork, node
                                    )
                                },
                                TestType::Memory => {
                                    format!(
                                        r#"{{"intensity": {}, "duration": {}, "size": {}, "node": "{}"}}"#,
                                        intensity, duration, size, node
                                    )
                                },
                                TestType::Disk => {
                                    format!(
                                        r#"{{"intensity": {}, "duration": {}, "size": {}, "node":"{}"}}"#,
                                        intensity, duration, size, node
                                    )
                                },
                            };
                        
                            let command = format!(
                                "curl -X POST {}/{} -H \"Content-Type:application/json\" -d '{}' ",
                                server_url, endpoint, payload
                            );
                        
                            println!("Executing: {}", command);
                            results.push(format!("Executing: {}", command));
                        
                            // Execute the curl command
                            let output = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&command)
                                .output();
                            
                            match output {
                                Ok(output) => {
                                    let status_str = if output.status.success() { "SUCCESS" } else { "FAILED" };
                                    results.push(format!("Status: {}", status_str));
                                    let stdout = String::from_utf8_lossy(&output.stdout);
                                    if !stdout.is_empty() {
                                        results.push(format!("Output: {}", stdout));
                                    }
                                    if !output.status.success() {
                                        let stderr = String::from_utf8_lossy(&output.stderr);
                                        results.push(format!("Error: {}", stderr));
                                    }
                                },
                                Err(e) => {
                                    results.push(format!("Failed to execute command: {}", e));
                                }
                            }
                        
                            results.push("---".to_string());
                        }
                        
                        // Return the result as a joined string
                        results.join("\n")
                    },
                    Message::TestComplete, // This will pass the result to the TestComplete message
                );
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Create collapsible advanced section that contains server URL
        let advanced_section = if self.show_advanced {
            let server_url_input = TextInput::new(
                "Server URL (e.g., http://localhost:8080)",
                &self.server_url,
            )
            .on_input(Message::ServerUrlChanged)
            .padding(10);
            
            Column::new()
                .push(server_url_input)
                .spacing(10)
        } else {
            Column::new()
        };
        
        // Advanced toggle button
        let advanced_toggle = Row::new()
            .push(Text::new("Advanced Settings").size(16))
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(toggler(
                Some(String::from("")),
                self.show_advanced,
                |_| Message::ToggleAdvanced,
            ))
            .spacing(10)
            .align_items(Alignment::Center);

        // Common parameters
        let duration_input = TextInput::new(
            "Duration (seconds)",
            &self.duration,
        )
        .on_input(Message::DurationChanged)
        .padding(10);

        let intensity_input = TextInput::new(
            "Intensity (threads)",
            &self.intensity,
        )
        .on_input(Message::IntensityChanged)
        .padding(10);

        let size_input = TextInput::new(
            "Size (MB)",
            &self.size,
        )
        .on_input(Message::SizeChanged)
        .padding(10);
       
        let load_input = TextInput::new(
            "CPU Load (%)",
            &self.load,
        )
        .on_input(Message::LoadChanged)
        .padding(10);
       
        // CPU fork toggle - using a checkbox instead of toggler for compatibility
        let fork_checkbox = Checkbox::new(
            "Enable Fork",
            self.fork,
            Message::ForkToggled,
        );

        let node_input = TextInput::new(
            "Node Name",
            &self.node,
        )
        .on_input(Message::NodeChanged)
        .padding(10);

        // Parameters in a row
        let row1 = Row::new()
            .push(duration_input)
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(intensity_input)
            .spacing(5);
           
        let row2 = Row::new()
            .push(size_input)
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(load_input)
            .push(Space::with_width(Length::Fixed(10.0)))
            .push(node_input)
            .spacing(5);
           
        // CPU-specific controls
        let cpu_controls = Row::new()
            .push(fork_checkbox)
            .spacing(5);

        // Test checkboxes
        let checkboxes = vec![
            (TestType::Cpu, "CPU Test"),
            (TestType::Memory, "Memory Test"),
            (TestType::Disk, "Disk Test"),
        ]
        .into_iter()
        .fold(Row::new().spacing(20), |row, (test, label)| {
            row.push(Checkbox::new(
                label,
                self.selected_tests.contains(&test),
                move |checked| Message::ToggleTest(test, checked),
            ))
        });

        // Helper text that explains test parameters
        let helper_text = Column::new()
            .push(Text::new("Test Parameter Information:").size(16))
            .push(Text::new("CPU Test: Uses intensity (threads) and duration"))
            .push(Text::new("Memory Test: Uses size (MB) and duration"))
            .push(Text::new("Disk Test: Uses intensity (threads) and duration"))
            .spacing(5);

        // Create a run button with loading state
        let run_button = if self.running_tests {
            Button::new(
                Text::new("RUNNING...").size(30)
            )
            .padding(20)
            .style(iced::theme::Button::Secondary)
            .width(Length::Fixed(200.0))
        } else {
            Button::new(
                Text::new("RUN TESTS").size(30)
            )
            .on_press(Message::RunPressed)
            .padding(20)
            .style(iced::theme::Button::Primary)
            .width(Length::Fixed(200.0))
        };

        // Status message display
        let status = match &self.status_message {
            Some(message) => {
                let scrollable = Scrollable::new(
                    Text::new(message).size(14)
                )
                .height(Length::Fixed(150.0));

                Column::new().push(scrollable)
            },
            None => Column::new(),
        };

        // Main content layout
        let content = Column::new()
            .push(Text::new("Mogwai Stress Tool").size(28))
            .push(advanced_toggle)
            .push(advanced_section)
            .push(Text::new("Test Parameters:").size(20))
            .push(row1)
            .push(row2)
            .push(cpu_controls)
            .push(Text::new("Select Tests:").size(20))
            .push(checkboxes)
            .push(helper_text)
            .push(run_button)
            .push(status)
            .align_items(Alignment::Center)
            .spacing(20);

        Container::new(content)
            .center_x()
            .width(Length::Fill)
            .padding(40)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        // Return none since we're not using subscriptions for timer
        iced::Subscription::none()
    }
}

pub fn run() -> iced::Result {
    GuiApp::run(Settings::default())
}
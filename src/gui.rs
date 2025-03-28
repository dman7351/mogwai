// Possible GUI integration for our stresser.
// From researching on how to create a GUI, "iced" is commmonly used and can be "Cross-platform", 
// Inspiration from "The Elm Architectures", It will have three parts such as state, view, update
// This possible GUI implemtation, will have boxes with labels and a run button that will run that test
// I would think it would be best if it just a direct call (tokio), to that code, so directly calling the test, and that being it.
// Stll in early production, will change, any suggestion appreciated.

use iced::{button, text_input, Button, Checkbox, Column, Element, Sandbox, Settings, Text, TextInput};
//use rand::Rng;
use std::sync::Arc;
use std::threads;
//use std::time::{Duration, Instant};

// mod these two stress rs, testing but eventually 
// the list of all stress test rs will be lsited

mod memory_sress;
mod cpu_stress;

// Initializing the State, represented by StressGUI. Defining all data that will be tracked.
// At start of GUI, options will all be defaulted. 
// Defining a struct which will contain the following fields.
#[derive(Default)]
struct StressGUI {

    cpu_select: bool,
    memory_select: bool,
    run_button: button::State,
    memory_size: usize,
    memory_time: u64,
    memory_size_input: text_input::State,
    memory_time_input: text_input::State,

}

// enum MyGUIMessage, here we are creating the types for the values found in our struct (events)
// This portion "MyGUIMessage" is how the applicaiton will respond to user action
// Interactions/Events and how they are to be handled. 
//
#[derive(Debug, Clone)]
enum MyGUIMessage{

    ToggleCPU(bool),
    ToggleMemory(bool),
    RunStressTest,
    MemorySizeChanged(usize),
    MemoryTimeChanged(u64),

}

// We are implementing a trait for struct StressGUI, and sandbox is a trait provided by iced that allows for gui app
// Sandbox expects a type called message, MyGUIMessage is what we are using for interactions
// Creating a new instance of StressGUI. derived from #[derive(Default)], sets checkboxes to false, and fields for memory
// title sets the title of the application window
// update receives messages and updates the gui state accordingly
impl Sandbox for StressGUI {
    type Message = MyGUIMessage;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Mogwai Stress Toolkit")

    }

    fn update(&mut self, message: Message){
        match message {
            MyGUIMessage::ToggleCPU(selected) => self.cpu_selected = selected,
            MyGUIMessage::ToggleMemory(selected) => self.memory_selected = selected,
            MyGUIMessage::MemorySizeChanged(size) => self.memory_size = size,
            MyGUIMessage::MemoryTimeChanged(time) => self.memory_time = time,
            MyGUIMessage::RunStressTest => {
                println!(
                    "Running stress test... CPU: {}, Memory: {}, (Size: {}, Time: {}), Disk: {}",
                    self.cpu_selected, self.memory_selected, self.memory_size, self.memory_time
                );


            }
        }

    }

}

// This code will not work at the moment. There is more that needs to be added!
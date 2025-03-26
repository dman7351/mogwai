// Possible GUI integration for our stresser.
// From researching on how to create a GUI, "iced" is commmonly used and can be "Cross-platform"
// Inspiration from "The Elm Architectures", It will have four concepts such as state, messages, view logic, update logic
// This possible GUI implemtation, will have boxes with labels and a run button that will run that test
// I would think it would be best if it just a dirrect call, to that code, so directly calling the test, and that being it.
// Stll in early production, will change, any suggestion appreciated.

use iced::{button, text_input, Button, Checkbox, Column, Element, Sandbox, Settings, Text, TextInput};
use rand::Rng;
use std::sync::Arc;
use std::threads;
use std::time::{Duration, Instant};

// "important" these two stress rs, testing but eventually 
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


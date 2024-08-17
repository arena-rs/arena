use std::collections::HashMap;
use plotly::{Plot, Scatter};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::fs::OpenOptions;

/// Trait allowing custom behavior to be defined for logging and inspecting values.
pub trait Inspector<V> {
    /// Log a value to state.
    fn log(&mut self, value: V);

    /// Inspect a value at a given time step.
    fn inspect(&self, step: usize) -> Option<V>;

    /// Save the inspector state.
    fn save(&self, save_type: Option<SaveData>);
}

pub enum SaveData {
    ToFile(String),
    ToNewFile(String),
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    id: usize, // Id of the log
    name: String, // Name of the log
    data: String, // The data being stored inside the log
}

impl LogMessage {
    pub fn new(name: String, data: String) -> Self {
        LogMessage {
            id: 0,
            name: name,
            data: data,
        }
    }

    pub fn update_counter(&mut self, counter: usize) {
        self.id = counter;
    }
}

#[derive(Debug)]
pub struct Logger {
    pub values: Vec<LogMessage>, // Stores all the messages
    pub counter: usize, // Used for keeping position of the id tag
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            values: Vec::new(),
            counter: 0,
        }
    }

    pub fn append_to_csv(file_location: &String, contents: String) {
        let path = Path::new(file_location); 
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(path)
            .unwrap();
    
        if let Err(e) = writeln!(file, "{}", contents) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }

    pub fn create_csv(file_location: &String) {
        let mut file = File::create(file_location);
    }
}

impl Inspector<LogMessage> for Logger {
    fn log(&mut self, mut value: LogMessage) {
        value.update_counter(self.counter);
        self.counter += 1;
        self.values.push(value);
    }

    fn inspect(&self, step: usize) -> Option<LogMessage> {
        Some(self.values[step].clone())
    }

    fn save(&self, save_type: Option<SaveData>) {
        let mut file_loc = String::new();
        match save_type.unwrap() { 
            SaveData::ToNewFile(file_location) => {
                Logger::create_csv(&file_location);
                file_loc = file_location;
            },
            SaveData::ToFile(file_location) => {
                file_loc = file_location;
            },
            _ => {
                panic!("Failed to get save_type enum");
            },
        }

        let intial_data = String::from("id;name;data");
        Logger::append_to_csv(&file_loc, intial_data);
        for log in self.values.clone() {
            let data = format!("{};{};{}", log.id, log.name, log.data);
            Logger::append_to_csv(&file_loc, data);
        }
    }
}

// pub struct Plotter {
//     values: Vec<f64>,
// }

// impl Plotter {
//     pub fn new() -> Self {
//         Plotter {
//             values: Vec::new(),
//         }
//     }
// }

// impl Inspector<f64> for Plotter {
//     fn log(&mut self, value: f64) {
//         self.values.push(value);
//     }

//     fn inspect(&self, step: usize) -> Option<f64> {
//         Some(self.values[step])
//     }

//     fn save(&self, save_type: Option<SaveData>) {
//         let mut plot = Plot::new();

//         let timesteps: Vec<usize> = (0..self.values.len()).collect();
//         let values: Vec<f64> = timesteps.iter().map(|v| v).collect();

//         let trace = Scatter::new(timesteps, values).mode(plotly::common::Mode::Markers);

//         plot.add_trace(trace);
//         plot.show();
//     }
// }

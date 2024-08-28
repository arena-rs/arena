use std::collections::HashMap;
use plotly::{Plot, Scatter};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::fs::OpenOptions;
use std::{error::Error, io, process};
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};

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
    ToCsv(String),
    ToNewCsv(String),
    ToJson(String),
    ToNewJson(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    fn append_to_csv(record: LogMessage, path: &Path) -> Result<(), csv::Error> {
        let file = OpenOptions::new() 
            .append(true)
            .create(true)
            .open(path)?;
    
        let mut writer = csv::Writer::from_writer(file);
    
        writer.serialize((
            record.id, 
            record.name, 
            record.data
        ))?;
        writer.flush()?;
    
        Ok(())
    }

    fn create_csv(file_location: &String) -> Result<(), csv::Error> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_location)?;
    
        let mut writer = csv::Writer::from_writer(file);
        writer.write_record(&["id", "name", "data"])?;
        writer.flush()?;
    
        Ok(())
    }

    fn create_json(file_location: &String) -> Result<(), Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_location)?;
    
        let empty_array = json!([]);
        serde_json::to_writer(file, &empty_array)?;
    
        Ok(())
    }
    
    fn append_to_json(record: LogMessage, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut records = Self::read_json_file(&path.to_string_lossy().to_string())?;
    
        records.push(record);
    
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
    
        serde_json::to_writer(file, &records)?;
    
        Ok(())
    }
    
    fn read_json_file(file_loc: &String) -> Result<Vec<LogMessage>, Box<dyn std::error::Error>> {
        let file = File::open(file_loc)?;
        let records: Vec<LogMessage> = serde_json::from_reader(file)?;
        Ok(records)
    }
    
    /*
    fn read_csv_file(file_loc: &String) -> Result<Vec<LogMessage>, csv::Error> {
        let mut records: Vec<LogMessage> = vec![];
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_loc)?;

        let mut reader = csv::Reader::from_reader(file);

        for record in reader.deserialize() {
            // println!("Line");
            let record: Record = record?;
            records.push(record);
        }
        Ok(records)
    }
    */
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
            SaveData::ToNewCsv(file_location) => {
                Logger::create_csv(&file_location);
                for log in self.values.clone() {
                    let file_path = Path::new(&file_location);
                    Logger::append_to_csv(log, &file_path);
                }
            },
            SaveData::ToCsv(file_location) => {
                for log in self.values.clone() {
                    let file_path = Path::new(&file_location);
                    Logger::append_to_csv(log, &file_path);
                }
            },
            
            SaveData::ToNewJson(file_location) => {
                Logger::create_json(&file_location);
                for log in self.values.clone() {
                    let file_path = Path::new(&file_location);
                    Logger::append_to_json(log, &file_path);
                }
            },

            SaveData::ToJson(file_location) => {
                for log in self.values.clone() {
                    let file_path = Path::new(&file_location);
                    Logger::append_to_json(log, &file_path);
                }
            }
        }
    }
}

/// No-op implementation of an [`Inspector`] for custom usecases.
pub struct EmptyInspector;

impl Inspector<f64> for EmptyInspector {
    fn inspect(&self, _step: usize) -> Option<f64> {
        None
    }
    fn log(&mut self, _value: f64) {}
    fn save(&self) {}
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

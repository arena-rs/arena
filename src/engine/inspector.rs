use std::{fs::OpenOptions, io::Seek};

use serde::{Deserialize, Serialize};

/// Trait allowing custom behavior to be defined for logging and inspecting values.
pub trait Inspector<V> {
    /// Log a value to state.
    fn log(&mut self, value: V);

    /// Inspect a value at a given time step.
    fn inspect(&self, step: usize) -> Option<V>;

    /// Save the inspector state.
    fn save(&self);
}

/// Type that allows for logging indexed values to files on disc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogMessage {
    /// Index of the log message.
    pub id: usize,

    /// Key of the log message.
    pub name: String,

    /// Data of the log message.
    pub data: String,
}

impl LogMessage {
    /// Public constructor function for a new [`LogMessage`].
    pub fn new(name: String, data: String) -> Self {
        Self { id: 0, name, data }
    }
}

#[derive(Debug)]
/// Custom implementation of an [`Inspector`] for logging values to a file (CSV or JSON).
pub struct Logger {
    values: Vec<LogMessage>,
    counter: usize,
    file_path: String,
    format: LogFormat,
}

#[derive(Debug)]
/// Enum to specify the logging format.
enum LogFormat {
    Csv,
    Json,
}

impl Logger {
    /// Public constructor function for a new [`Logger`] for CSV format.
    pub fn new_csv(file_path: String) -> Self {
        Self {
            values: Vec::new(),
            counter: 0,
            file_path,
            format: LogFormat::Csv,
        }
    }

    /// Public constructor function for a new [`Logger`] for JSON format.
    pub fn new_json(file_path: String) -> Self {
        Self {
            values: Vec::new(),
            counter: 0,
            file_path,
            format: LogFormat::Json,
        }
    }

    /// Append a log message to the appropriate file format.
    fn append_to_file(&self, record: &LogMessage) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.file_path)?;

        match self.format {
            LogFormat::Csv => {
                let mut writer = csv::Writer::from_writer(file);
                writer.serialize((record.id, &record.name, &record.data))?;
                writer.flush()?;
            }
            LogFormat::Json => {
                let mut records: Vec<LogMessage> = if file.metadata()?.len() > 0 {
                    serde_json::from_reader(&file)?
                } else {
                    Vec::new()
                };
                records.push(record.clone());
                file.set_len(0)?;
                file.seek(std::io::SeekFrom::Start(0))?;
                serde_json::to_writer_pretty(file, &records)?;
            }
        }
        Ok(())
    }
}

impl Inspector<LogMessage> for Logger {
    fn log(&mut self, mut value: LogMessage) {
        value.id = self.counter;
        self.counter += 1;
        self.values.push(value.clone());

        if let Err(e) = self.append_to_file(&value) {
            eprintln!("Failed to append to file: {}", e);
        }
    }

    fn inspect(&self, step: usize) -> Option<LogMessage> {
        self.values.get(step).cloned()
    }

    fn save(&self) {}
}

/// No-op implementation of an [`Inspector`] for custom use cases.
pub struct EmptyInspector;

impl Inspector<f64> for EmptyInspector {
    fn inspect(&self, _step: usize) -> Option<f64> {
        None
    }
    fn log(&mut self, _value: f64) {}
    fn save(&self) {}
}

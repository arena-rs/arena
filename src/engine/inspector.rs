// use plotly::{Plot, Scatter};

/// Trait allowing custom behavior to be defined for logging and inspecting values.
pub trait Inspector<V> {
    /// Log a value to state.
    fn log(&mut self, value: V);

    /// Inspect a value at a given time step.
    fn inspect(&self, step: usize) -> Option<V>;

    /// Save the inspector state.
    fn save(&self);
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
//     values: Vecf64>,
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

//     fn save(&self) {
//         let mut plot = Plot::new();

//         let timesteps: Vec<usize> = (0..self.values.len()).collect();
//         let values: Vec<f64> = timesteps.iter().map(|v| v).collect();

//         let trace = Scatter::new(timesteps, values).mode(plotly::common::Mode::Markers);

//         plot.add_trace(trace);
//         plot.show();
//     }
// }


use std::collections::BTreeMap;

extern crate chrono;
use self::chrono::Local;

#[derive(Clone)]
pub enum Color {
    /// An HTML color (#1234aa)
    Arbitrary(String),
    /// The default
    Standard,
    /// Various colors
    White,
    Red,
    Green,
    Blue,
    Orange,
    Purple,
}

/// An output produced by a metric to be displayed in the bar.
pub struct RenderResult {
    pub text: String,
    pub color: Color,
}

/// State that is stored in a MetricState between rendering cycles.
#[derive(Clone)]
pub enum State {
    Empty,
    S(String),
    I(i64),
    F(f64),
    C(Color),
}

/// State that is passed to and returned from every render cycle.
pub struct MetricState {
    /// Arbitrary state
    state: BTreeMap<String, State>,

    /// Unix epoch in seconds. This is updated by the framework.
    pub last_called: i64,
}

impl MetricState {
    pub fn new() -> MetricState {
        MetricState {
            state: BTreeMap::new(),
            last_called: 0,
        }
    }
    pub fn get(&self, k: String) -> State {
        self.state.get(&k).unwrap_or(&State::Empty).clone()
    }
    pub fn set(&mut self, k: String, v: State) {
        self.state.insert(k, v);
    }
    pub fn now() -> i64 {
        Local::now().timestamp()
    }
}

pub trait Metric {
    /// Initializes a metric using the string supplied as parameter to the command line argument.
    fn init(&self, argvalue: Option<String>) -> MetricState;
    /// Renders the metric.
    fn render(&mut self, st: &mut MetricState) -> RenderResult;
}

/// A metric that is active in the current run and updated for every cycle.
pub struct ActiveMetric {
    name: String,
    m: Box<Metric>,
    st: MetricState,
}

impl ActiveMetric {
    pub fn new(name: String, metric: Box<Metric>, initial_state: MetricState) -> ActiveMetric {
        ActiveMetric {
            name: name,
            m: metric,
            st: initial_state
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}


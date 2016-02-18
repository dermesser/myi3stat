#![allow(dead_code)]

extern crate chrono;
use self::chrono as chron;

#[derive(Clone)]
pub enum Color {
    /// An HTML color (#1234aa)
    Arbitrary(String),
    Default,
    /// Various colors
    White,
    Red,
    Green,
    Blue,
    Black,
    Orange,
    Purple,
}

impl Color {
    fn to_string(&self) -> String {
        match self.clone() {
            Color::Arbitrary(c) => c,
            Color::Default => String::from("#ffffff"),
            Color::White => String::from("#ffffff"),
            Color::Red => String::from("#ff0000"),
            Color::Green => String::from("#00ff00"),
            Color::Blue => String::from("#0000ff"),
            Color::Black => String::from("#000000"),
            Color::Orange => String::from("#e8a317"),
            Color::Purple => String::from("#8d0552"),
        }
    }
}

/// An output produced by a metric to be displayed in the bar.
pub struct RenderResult {
    name: String,
    text: String,
    color: Color,
}

impl RenderResult {
    pub fn new(text: String, color: Color) -> RenderResult {
        RenderResult {
            name: String::new(),
            text: text,
            color: color,
        }
    }
    pub fn to_json(&self) -> String {
        let result = format!("{{\"name\": \
                              \"{name}\",\"color\":\"{color}\",\"markup\":\"none\",\"full_text\":\
                              \"{text}\"}}",
                             name = self.name,
                             color = self.color.to_string(),
                             text = self.text);
        result
    }
}

/// State that is passed to and returned from every render cycle.
pub struct MetricState {
    /// Unix epoch in seconds. This is updated by the framework.
    pub last_called: i64,
}

impl MetricState {
    pub fn new() -> MetricState {
        MetricState { last_called: 0 }
    }
    /// Returns timestamp in epoch milliseconds.
    pub fn now() -> i64 {
        use self::chrono::Timelike;
        let t = chron::Local::now();
        1000 * t.timestamp() + (t.nanosecond() as i64 / 1000000)
    }
}

pub trait Metric {
    /// Initializes a metric using the string supplied as parameter to the command line argument.
    fn init(&mut self, _: &mut MetricState, _: Option<String>) {}
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
            st: initial_state,
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn render(&mut self) -> RenderResult {
        let mut result = self.m.render(&mut self.st);
        self.st.last_called = MetricState::now();
        result.name = self.name.clone();
        result
    }
}

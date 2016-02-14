#![allow(dead_code)]

use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;

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
    Black,
    Orange,
    Purple,
}

impl Color {
    fn to_string(&self) -> String {
        match self.clone() {
            Color::Arbitrary(c) => c,
            Color::Standard => String::from("#ffffff"),
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
    pub text: String,
    pub color: Color,
}

impl RenderResult {
    fn to_json(&self) -> String {
        let result = format!(
            "{{\"name\": \"{name}\",\"color\":\"{color}\",\"markup\":\"none\",\"full_text\":\"{text}\"}}",
            name=self.name,
            color=self.color.to_string(),
            text=self.text);
        result
    }
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
            st: initial_state,
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn render(&mut self) -> RenderResult {
        let mut result = self.m.render(&mut self.st);
        result.name = self.name.clone();
        result
    }
}

pub fn render_loop(mut metrics: Vec<ActiveMetric>, interval: i32) {
    let ival_duration = Duration::new(interval as u64, 0);
    let intro = "{\"version\":1}\n[\n";
    println!("{}", intro);

    loop {
        let render_result = metrics.iter_mut().map(|m| m.render()).fold(String::from(""), |mut out, p| { out.push_str(&p.to_json()); out.push_str(","); out });

        println!("[{}],", render_result);

        sleep(ival_duration);
    }
}

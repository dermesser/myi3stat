use std::collections::BTreeMap;
use std::process;
use std::env;

extern crate chrono;
use chrono::Local;

extern crate getopts;
use getopts::Options;

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

trait Metric {
    /// Initializes a metric using the string supplied as parameter to the command line argument.
    fn init(&self, argvalue: Option<String>) -> MetricState;
    /// Renders the metric.
    fn render(&mut self, st: &mut MetricState) -> RenderResult;
}

/// A metric that is active in the current run and updated for every cycle.
struct ActiveMetric {
    name: String,
    m: Box<Metric>,
    st: MetricState,
}

/// Represents a/the set of metrics available for display.
struct AvailableMetrics {
    metrics: BTreeMap<String, Box<Metric>>,
    opts: Options,
}

impl AvailableMetrics {
    fn new() -> AvailableMetrics {
        let mut options = Options::new();
        options.optopt("",
                       "ordering",
                       "Ordering of metrics in status bar. Comma-separated list of metric names \
                        (default ordering is ASCII-ordered, i.e. case sensitive)",
                       "METRIC1,METRIC2,METRIC3");
        options.optopt("",
                       "interval",
                       "Interval in seconds between individual render cycles. Default: 1",
                       "SECONDS");
        options.optflag("h", "help", "Print a help text");

        AvailableMetrics {
            metrics: BTreeMap::new(),
            opts: options,
        }
    }

    /// Register a metric under the given name.
    /// desc and example are for the purpose of documenting the command line option that is added.
    /// Does 
    fn register_metric(&mut self, name: &str, desc: &str, example: &str, metric: Box<Metric>) {
        if !self.metrics.contains_key(&String::from(name)) {
            self.opts.optopt("", name, desc, example);
            self.metrics.insert(String::from(name), metric);
        }
    }

    fn print_help(&self) {
        print!("{}", self.opts.usage("Usage: myi3stat [options]"));
    }

    fn parse_args(&mut self, args: &[String]) -> getopts::Matches {
        let matches = self.opts.parse(args);

        match matches {
            Err(_) => {
                self.print_help();
                process::exit(1)
            }
            Ok(m) => m,
        }
    }

    /// Returns a map of metric -> position in ordering list
    fn make_ordering_map(ord_list: String) -> BTreeMap<String, i32> {
        let parts = ord_list.split(",");
        let mut i = 0;
        let mut ordmap = BTreeMap::new();

        for metric in parts {
            ordmap.insert(String::from(metric), i);
            i += 1;
        }
        ordmap
    }

    /// Returns a vec of the selected metrics in the wanted order, and the interval between render
    /// cycles in seconds.
    fn evaluate(mut self, args: &[String]) -> (Vec<ActiveMetric>, i32) {
        let matches = self.parse_args(args);
        let mut metrics = Vec::new();

        // Look for every defined metric if the user wants to have it displayed.
        for (metric_name, metric) in self.metrics.into_iter() {
            if matches.opt_present(&metric_name) {
                let st = metric.init(matches.opt_str(&metric_name));
                metrics.push(ActiveMetric {
                    name: String::from(metric_name),
                    m: metric,
                    st: st,
                });
            }
        }

        // Sort metrics by position in the supplied ordering list (or alternatively
        // alphabetically).
        let ordmap = AvailableMetrics::make_ordering_map(matches.opt_str("ordering")
                                                                .unwrap_or(String::from("")));
        metrics.sort_by(|a, b| {
            match (ordmap.get(&a.name), ordmap.get(&b.name)) {
                (Some(i1), Some(i2)) => i1.cmp(i2),
                (_, _) => a.name.cmp(&b.name),
            }
        });

        let interval = i32::from_str_radix(&matches.opt_str("interval")
                                                   .unwrap_or(String::from("1")),
                                           10)
                           .unwrap_or(1);

        (metrics, interval)
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let all_metrics = AvailableMetrics::new();
    let (selected_metrics, interval) = all_metrics.evaluate(&args[1..]);
}

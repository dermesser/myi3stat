mod framework;
mod helper;
mod metrics;

use std::collections::BTreeMap;
use std::process;
use std::env;

extern crate getopts;
use getopts::Options;

use framework::*;

/// Represents a/the set of metrics available for display.
struct AvailableMetrics {
    metrics: BTreeMap<String, Box<Metric>>,
    opts: Options,
}


/// Set of all metrics. Used to register metrics and select the active ones based on the user's
/// selection.
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
    fn register_metric(&mut self, name: &str, desc: &str, example: &str, metric: Box<Metric>) {
        if !self.metrics.contains_key(&String::from(name)) {
            self.opts.optflagopt("", name, desc, example);
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
            Ok(m) => {
                if m.opt_present("help") || m.opt_present("h") {
                    self.print_help();
                    process::exit(1)
                }
                m
            }
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
                let mut st = MetricState::new();
                metric.init(&mut st, matches.opt_str(&metric_name));
                metrics.push(ActiveMetric::new(String::from(metric_name), metric, st));
            }
        }

        // Sort metrics by position in the supplied ordering list (or alternatively
        // alphabetically).
        let ordmap = AvailableMetrics::make_ordering_map(matches.opt_str("ordering")
                                                                .unwrap_or(String::from("")));
        metrics.sort_by(|a, b| {
            match (ordmap.get(a.name()), ordmap.get(b.name())) {
                (Some(i1), Some(i2)) => i1.cmp(i2),
                (_, _) => a.name().cmp(b.name()),
            }
        });

        let interval = i32::from_str_radix(&matches.opt_str("interval")
                                                   .unwrap_or(String::from("1")),
                                           10)
                           .unwrap_or(1);

        (metrics, interval)
    }
}

fn register_metrics(registry: &mut AvailableMetrics) {
    use metrics::time;

    // List of codes: https://lifthrasiir.github.io/rust-chrono/chrono/format/strftime/index.html
    registry.register_metric("clock",
                             "A timestamp clock. Uses format codes like date(1)",
                             "%H:%M",
                             time::clock_metric());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut all_metrics = AvailableMetrics::new();
    register_metrics(&mut all_metrics);
    let (selected_metrics, interval) = all_metrics.evaluate(&args[1..]);

    render_loop(selected_metrics, interval);
}

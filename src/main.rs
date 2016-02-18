mod framework;
mod helper;
mod metrics;
mod render;

use std::collections::BTreeMap;
use std::process;
use std::env;

extern crate getopts;
use getopts::Options;

use framework::*;
use render::*;

/// Represents a/the set of metrics available for display.
struct Config {
    metrics: BTreeMap<String, Box<Metric>>,
    renderers: BTreeMap<String, Box<Renderer>>,
    opts: Options,
}


/// Set of all metrics. Used to register metrics and select the active ones based on the user's
/// selection.
impl Config {
    fn new() -> Config {
        let mut options = Options::new();
        options.optopt("",
                       "ordering",
                       "Ordering of metrics in status bar. Comma-separated list of metric names \
                        (default ordering is ASCII-ordered, i.e. case sensitive)",
                       "METRIC1,METRIC2,METRIC3");
        options.optopt("",
                       "interval",
                       "Interval in milliseconds between individual render cycles. Default: 1000",
                       "SECONDS");
        options.optopt("",
                       "renderer",
                       "Which renderer to use. Currently available: i3status,plain",
                       "i3status");
        options.optflag("h", "help", "Print a help text");

        Config {
            metrics: BTreeMap::new(),
            renderers: BTreeMap::new(),
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

    fn register_renderer(&mut self, name: &str, r: Box<Renderer>) {
        if !self.renderers.contains_key(name) {
            self.renderers.insert(String::from(name), r);
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
        let mut i = 1;
        let mut ordmap = BTreeMap::new();

        for metric in parts {
            ordmap.insert(String::from(metric), i);
            i += 1;
        }
        ordmap
    }

    /// Returns the selected renderer; the list of selected metrics; and the chosen interval in
    /// milliseconds.
    fn evaluate(mut self, args: &[String]) -> (Box<Renderer>, Vec<ActiveMetric>, i32) {
        use std::str::FromStr;

        let matches = self.parse_args(args);
        let mut metrics = Vec::new();

        // Look for every defined metric if the user wants to have it displayed.
        for (metric_name, mut metric) in self.metrics.into_iter() {
            if matches.opt_present(&metric_name) {
                let mut st = MetricState::new();
                metric.init(&mut st, matches.opt_str(&metric_name));
                metrics.push(ActiveMetric::new(String::from(metric_name), metric, st));
            }
        }

        // Sort metrics by position in the supplied ordering list (or alternatively
        // alphabetically).
        let ordmap = Config::make_ordering_map(matches.opt_str("ordering")
                                                      .unwrap_or(String::from("")));
        metrics.sort_by(|a, b| {
            match (ordmap.get(a.name()), ordmap.get(b.name())) {
                (Some(i1), Some(i2)) => i1.cmp(i2),
                (_, _) => a.name().cmp(b.name()),
            }
        });

        // Select and set up renderer
        let interval = i32::from_str(&matches.opt_str("interval").unwrap_or(String::from("1000")))
                           .unwrap_or(1000);
        let renderer_name = matches.opt_str("renderer").unwrap_or(String::from("i3status"));


        if !self.renderers.contains_key(&renderer_name) {
            panic!(format!("Renderer '{}' not registered!!", renderer_name));
        }

        let renderer = self.renderers.remove(&renderer_name).unwrap();

        (renderer, metrics, interval)
    }
}

fn register_metrics(registry: &mut Config) {
    use metrics::cpu_load;
    use metrics::load;
    use metrics::net;
    use metrics::time;

    // List of codes: https://lifthrasiir.github.io/rust-chrono/chrono/format/strftime/index.html
    registry.register_metric("clock",
                             "A timestamp clock. Uses format codes like date(1)",
                             "%H:%M",
                             time::clock_metric());
    registry.register_metric("netif",
                             "Shows total received/transmitted bytes for network interaces",
                             "eth0,lo",
                             net::make_net_metric());
    registry.register_metric("load",
                             "Shows the last three load averages over the last (1, 5, 15) \
                              minutes.",
                             "",
                             load::make_load_metric());
    registry.register_metric("cpu_load",
                             "Shows the cpu load in percent over the last measure interval. abs \
                              means: 4 core seconds = 400%; rel means (on a quadcore): 4 core \
                              seconds = 100%",
                             "abs|rel",
                             cpu_load::make_cpu_load_metric());
}

fn register_renderers(registry: &mut Config) {
    use render;

    registry.register_renderer("i3status", render::make_i3status());
    registry.register_renderer("plain", render::make_plaintextrenderer());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cfg = Config::new();
    register_metrics(&mut cfg);
    register_renderers(&mut cfg);
    let (renderer, selected_metrics, interval) = cfg.evaluate(&args[1..]);


    render_loop(renderer, selected_metrics, interval);
}

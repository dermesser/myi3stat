//! A simple clock.

use framework::*;

extern crate time;
//use self::time;

struct TimeMetric {
    fmt: String,
}

const DEFAULT_FMT: &'static str = "%a %b %d %H:%M:%S %Y";

impl Metric for TimeMetric {
    fn init(&mut self, _: &mut MetricState, arg: Option<String>) {
        self.fmt = arg.unwrap_or(String::from(DEFAULT_FMT));
    }
    fn render(&mut self, _: &mut MetricState) -> RenderResult {
        let t = time::now();
        let tstr = format!("{}", t.strftime(&self.fmt).unwrap());

        RenderResult::new(tstr, Color::Default)
    }
}

pub fn clock_metric() -> Box<Metric> {
    Box::new(TimeMetric { fmt: String::new() })
}

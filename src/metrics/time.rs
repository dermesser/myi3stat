//! A simple clock.

use framework::*;

extern crate chrono;
use self::chrono::Local;

struct TimeMetric;

const DEFAULT_FMT: &'static str = "%a %b %d %H:%M:%S %Y (%Z)";

impl Metric for TimeMetric {
    fn init(&self, st: &mut MetricState, arg: Option<String>) {
        st.set(String::from("format"),
               State::S(arg.unwrap_or(String::from(DEFAULT_FMT))));
    }
    fn render(&self, st: &mut MetricState) -> RenderResult {
        let fmt;

        match st.get(String::from("format")) {
            State::S(f) => fmt = f,
            _ => fmt = String::from(DEFAULT_FMT),
        }

        let t = Local::now();
        let tstr = format!("{}", t.format(&fmt));

        RenderResult::new(tstr, Color::Default)
    }
}

pub fn clock_metric() -> Box<Metric> {
    Box::new(TimeMetric)
}
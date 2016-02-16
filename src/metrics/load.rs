use std::str::FromStr;

extern crate regex;
use self::regex::Regex;

use framework::*;
use helper::read_procfs_file;

struct LoadAvg;

impl LoadAvg {
    fn read_load_avg() -> (String, Color) {
        let loads = read_procfs_file(String::from("/loadavg"))
                        .unwrap_or(String::from("0.0 0.0 0.0  "));
        let re = Regex::new(r"([0-9\.]+)\s+([0-9\.]+)\s+([0-9\.]+).*").unwrap();

        match re.captures(&loads) {
            None => (String::from("0.0 0.0 0.0"), Color::Purple),
            Some(caps) => {
                (format!("{} {} {}",
                         caps.at(1).unwrap(),
                         caps.at(2).unwrap(),
                         caps.at(3).unwrap()),
                 LoadAvg::get_color(caps.at(1).unwrap()))
            }
        }
    }

    // load is a string containing one float number.
    fn get_color(load: &str) -> Color {
        let f = f64::from_str(load).unwrap_or(0.);

        // TODO: Make color thresholds configurable
        if f >= 0. && f < 1.5 {
            Color::Green
        } else if f >= 1.5 && f < 3. {
            Color::Orange
        } else if f >= 3. {
            Color::Red
        } else {
            Color::Default
        }
    }
}

impl Metric for LoadAvg {
    fn render(&mut self, _: &mut MetricState) -> RenderResult {
        let (loads, color) = LoadAvg::read_load_avg();
        RenderResult::new(loads, color)
    }
}

pub fn make_load_metric() -> Box<Metric> {
    Box::new(LoadAvg)
}

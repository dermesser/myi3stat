
extern crate regex;
use self::regex::Regex;

use framework::*;
use helper::read_procfs_file;
use helper::extract_from_str;

struct LoadAvg;

impl LoadAvg {
    fn read_load_avg() -> (String, Color) {
        let loads = read_procfs_file(String::from("/loadavg"))
                        .unwrap_or(String::from("0.0 0.0 0.0  "));
        let re = Regex::new(r"([0-9\.]+)\s+([0-9\.]+)\s+([0-9\.]+).*").unwrap();
        let load_avgs: Vec<f64> = extract_from_str(&loads, &re, 0.);

        if load_avgs.len() < 3 {
            (String::from("0.0 0.0 0.0"), Color::Purple)
        } else {
            (format!("{:5.2} {:5.2} {:5.2}",
                     load_avgs[0],
                     load_avgs[1],
                     load_avgs[2]),
             LoadAvg::get_color(load_avgs[0]))
        }
    }

    // load is a string containing one float number.
    fn get_color(f: f64) -> Color {

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

use framework::*;
use helper::{get_procfs_file_lines, extract_from_str};

extern crate regex;
use self::regex::Regex;

extern crate libc;

#[derive(PartialEq)]
enum DisplayMode {
    Absolute,
    Relative,
}

struct CPULoadMetric {
    mode: DisplayMode,
    ncpu: i32,
    user_hz: u64,
    last_cpu_millis: u64,

    individual_cpu_regex: Regex,
    total_cpu_regex: Regex,
}

impl CPULoadMetric {
    fn new() -> CPULoadMetric {
        use self::libc::{sysconf, _SC_CLK_TCK};

        CPULoadMetric {
            mode: DisplayMode::Absolute,
            ncpu: 1,
            user_hz: unsafe { sysconf(_SC_CLK_TCK) } as u64,
            last_cpu_millis: 0,
            individual_cpu_regex: Regex::new(r"^cpu(\d+)\s+").unwrap(),
            // user nice system
            total_cpu_regex: Regex::new(r"^cpu\s+(\d+)\s+(\d+)\s+(\d+)").unwrap(),
        }
    }
    fn get_number_of_cores(&self) -> i32 {
        let lines = get_procfs_file_lines(String::from("/stat"));

        match lines {
            None => 1,
            Some(lns) => {
                let mut n = 0;
                for line in lns {
                    if self.individual_cpu_regex.is_match(&line) {
                        n += 1;
                    } else if n > 0 {
                        break;
                    }
                }
                n
            }
        }
    }
    fn get_total_cpu_millis(&self) -> u64 {
        let lines = get_procfs_file_lines(String::from("/stat")).unwrap_or(Vec::new());

        for line in lines {
            let nums: Vec<u64> = extract_from_str(&line, &self.total_cpu_regex, 0);
            if nums.len() < 3 {
                continue;
            }
            return self.calc_total_cpu_millis(nums[0], nums[1], nums[2]);
        }
        return 0;
    }

    fn calc_total_cpu_millis(&self, user: u64, nice: u64, sys: u64) -> u64 {
        return (1000 / self.user_hz) * (user + nice + sys);
    }
}

impl Metric for CPULoadMetric {
    // arg can be "abs" or "rel" (default is 'abs')
    // "abs" means that a fully loaded CPU has a load of #cores * 100%
    // "rel" means that a fully loaded CPU has a load of 100%.
    fn init(&mut self, _: &mut MetricState, arg: Option<String>) {
        match arg {
            None => (),
            Some(s) => {
                if s == "rel" {
                    self.mode = DisplayMode::Relative;
                }
            }
        }
        self.ncpu = self.get_number_of_cores();
    }
    fn render(&mut self, st: &mut MetricState) -> RenderResult {
        // evaluation interval in milliseconds
        let interval = MetricState::now() - st.last_called;
        let current_time = self.get_total_cpu_millis();
        let diff = current_time - self.last_cpu_millis;
        self.last_cpu_millis = current_time;

        let mut percentage = 100f64 * (diff as f64 / interval as f64);

        if self.mode == DisplayMode::Relative {
            percentage /= self.ncpu as f64;
        }

        RenderResult::new(format!("{:4.0}%", percentage), Color::Default)
    }
}

pub fn make_cpu_load_metric() -> Box<Metric> {
    Box::new(CPULoadMetric::new())
}

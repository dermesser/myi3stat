use framework::*;
use helper::{commaseparated_to_vec, get_procfs_file_lines, extract_from_str};

extern crate regex;
use self::regex::Regex;

use std::collections::BTreeMap;

// Interface name, transmitted bytes, received bytes. Used for both rates and counters!
type IFStat = (String, u64, u64);

struct NetInterfaceMetric {
    oldstat: BTreeMap<String, IFStat>,
    whole_line_re: Regex,
    interface_re: Regex,
}

impl NetInterfaceMetric {
    fn new() -> NetInterfaceMetric {
        NetInterfaceMetric { oldstat: BTreeMap::new(),
        whole_line_re: Regex::new(r"^\s*[a-z0-9]+:\s+(\d+)\s+(\d+)\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s+(\d+)\s+(\d+)\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+$").unwrap(),
        interface_re: Regex::new(r"^\s*([a-z0-9]+):.+").unwrap()}
    }
    /// Obtain current counters from /proc/net/dev
    fn get_stats(&self, ifs: &BTreeMap<String, IFStat>) -> Vec<IFStat> {
        let ifstats;
        let mut processed_stats = Vec::with_capacity(ifs.len());

        match get_procfs_file_lines(String::from("/net/dev")) {
            None => ifstats = Vec::new(),
            Some(st) => ifstats = st,
        }

        //               RX                                                             TX
        //           *           *                                                    *         *
        //  iface |bytes       packets  errs drop fifo frame   compressed multicast|bytes     packets errs drop fifo colls   carrier compressed
        //  eth0:  1037503524  872642    0    0    0     0          0     10482     40971427  300143    0    1    0     0       0          0

        for line in ifstats {
            let results: Vec<String> = extract_from_str(&line,
                                                        &self.interface_re,
                                                        String::from(""));

            if results.len() < 1 {
                continue;
            }

            let interface = &results[0];

            if ifs.contains_key(interface) {
                let stats: Vec<u64> = extract_from_str(&line, &self.whole_line_re, 0);
                assert!(stats.len() > 3);
                processed_stats.push((interface.clone(), stats[0], stats[2]));

            }
        }

        processed_stats
    }

    /// Convert a number into a string with nice unit
    fn make_nice_rate(i: u64) -> String {
        let units = vec!["", "K", "M", "G"];
        let mut u = 0;
        let mut f = i as f64;
        loop {
            if f / 1024. > 1. {
                f = f / 1024.;
                u += 1;
            } else {
                break;
            }
        }
        assert!(u < 4);
        format!("{:6.1}{:1}", f, units[u])
    }

    /// Format a series of IFStat tuples
    fn format_stats(stats: Vec<IFStat>) -> String {
        stats.into_iter()
             .fold(String::new(), |mut acc, (i, rx, tx)| {
                 acc.push_str(&format!("{}: rx:{} tx:{} ",
                                       i,
                                       NetInterfaceMetric::make_nice_rate(rx),
                                       NetInterfaceMetric::make_nice_rate(tx)));
                 acc
             })
    }
}

impl Metric for NetInterfaceMetric {
    fn init(&mut self, _: &mut MetricState, initarg: Option<String>) {
        match initarg {
            None => (),
            Some(s) => {
                let mut wanted_ifs = BTreeMap::new();
                for intf in commaseparated_to_vec(s) {
                    wanted_ifs.insert(intf.clone(), (intf, 0, 0));
                }
                self.oldstat = wanted_ifs;
            }
        }
    }

    fn render(&mut self, st: &mut MetricState) -> RenderResult {
        let interval = (MetricState::now() - st.last_called) as u64;

        if self.oldstat.is_empty() {
            return RenderResult::new(String::from("n/a"), Color::Red);
        }

        // Get current counters
        let newstats = self.get_stats(&self.oldstat);
        let mut rates: Vec<IFStat> = Vec::new(); // this is the final output

        for (ifname, rx, tx) in newstats {
            // Obtain previous rx/tx counts from state
            let oldstat;
            match self.oldstat.get(&ifname) {
                Some(o) => oldstat = o.clone(),
                _ => oldstat = (ifname.clone(), rx, tx),
            }

            // calculate rate over last interval
            match oldstat {
                (ref ifname, oldrx, oldtx) => {
                    rates.push((ifname.clone(),
                                1000 * (rx - oldrx) / interval,
                                1000 * (tx - oldtx) / interval));
                }
            }
            // Store current counters in state
            self.oldstat.insert(ifname.clone(), (ifname, rx, tx));
        }

        RenderResult::new(NetInterfaceMetric::format_stats(rates), Color::Green)
    }
}

pub fn make_net_metric() -> Box<Metric> {
    Box::new(NetInterfaceMetric::new())
}

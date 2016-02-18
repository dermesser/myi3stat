use framework::*;

pub trait Renderer {
    fn init(&mut self, metrics: Vec<ActiveMetric>) -> String;
    fn render(&mut self) -> String;
}

struct I3statRenderer {
    metrics: Vec<ActiveMetric>,
}

impl I3statRenderer {
    fn new() -> I3statRenderer {
        I3statRenderer { metrics: Vec::new() }
    }
}

impl Renderer for I3statRenderer {
    fn init(&mut self, metrics: Vec<ActiveMetric>) -> String {
        self.metrics = metrics;

        String::from("{\"version\":1}\n[[]\n")
    }

    fn render(&mut self) -> String {
        let mut render_result = self.metrics
                                    .iter_mut()
                                    .map(|m| m.render())
                                    .fold(String::from(""), |mut out, rendres| {
                                        out.push_str(&rendres.to_json());
                                        out.push_str(",");
                                        out
                                    });
        render_result.pop();
        format!(",[{}]", render_result)
    }
}

pub fn make_i3status() -> Box<Renderer> {
    Box::new(I3statRenderer::new())
}

pub fn render_loop(mut r: Box<Renderer>, metrics: Vec<ActiveMetric>, interval: i32) {
    use std::thread::sleep;
    use std::time::Duration;

    let ival_duration = Duration::new((interval / 1000) as u64, 1000000 * (interval as u32 % 1000));

    print!("{}", r.init(metrics));

    loop {
        println!("{}", r.render());
        sleep(ival_duration);
    }
}

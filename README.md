# myi3stat

Intended to be a replacement for i3status, myi3stat is a small framework to
easily extend your i3 status bar. It is configured mainly via command line
arguments, and can be extended by small custom metrics.

## How to add your metric

### Define a metric type

Create a new module under `src/metrics/`, and define your type:

    struct MyCustomMetric;

You usually don't need any data members, as state between runs is carried in a
framework-supplied `MetricState` container.

### Implement the `Metric` trait

Implement the `Metric` trait for your type.

    pub trait Metric {
            fn init(&self, st: &mut MetricState, initarg: Option<String>);
            fn render(&self, st: &mut MetricState) -> RenderResult;
    }

The `init()` method takes a `MetricState` object, and an optional argument. The
argument is a user-supplied string from the command line invocation (see below,
"Register your metric"). `MetricState` is a quite simple type:

    // use framework::*;
    // Essentially a map of arbitrary keys to dynamically-typed State entries (see below).
    impl MetricState {
        pub fn get(&self, k: String) -> State;
        pub fn set(&mut self, k: String, v: State);
        // Obtain current timestamp.
        pub fn now() -> i64;
        // Time (Unix epoch) of last call.
        pub last_called: i64;
    }
    pub enum State {
        Empty,
        S(String),
        I(i64),
        F(f64),
        C(Color),
    }

Every time your metric is asked to `render()`, it is given the same
`MetricState` object; `last_called` is set to the timestamp of the previous
invocation (so you can compute a rate, for example); it has second-resolution.
You can arbitrarily get and set values on the `MetricState`.

Typically you will set some constant configuration parameters at the invocation
of your `init()` method, and use them later to determine how exactly you'll
render the metric.

At the end of your `render()` implementation, you return a `RenderResult`:

    RenderResult::new(contents, Color::Red)

This is the actual output that will appear.

Finally, export a factory function that creates an instance of your metric:

    pub fn make_your_metric() -> Box<Metric> { Box::new(MyCustomMetric) }

### Register your new metric

In `src/main.rs`, you need to register your metric inside the
`register_metrics()` function near the end of the file:

    use metric::your_metric;
    registry.register_metric("your_metric",
            "A metric that shows how frobnicated your foos are",  // Short description
            "format string",  // config parameter description
            your_metric::make_your_metric());  // An instance of your metric, of type Box<Metric>

This will add a command line flag that can be specified by users to activate
your metric, i.e., make it show up. There will also be a snippet of
documentation that is displayed when the user calls for `--help`:

    --your_metric [format string] A metric that shows how frobnicated your foos are

If a user invokes `myi3stat` like this, your new metric will be displayed:

    myi3stat --your_metric "%f %f %d"


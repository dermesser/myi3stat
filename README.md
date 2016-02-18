# myi3stat

Intended to be a replacement for i3status, myi3stat is a small framework to
easily extend your i3 status bar. It is configured mainly via command line
arguments, and can be extended by small custom metrics.

To view available metrics, execute

    myi3stat --help

## How to add your metric

(there are already some predefined metric types in src/metrics/ that you can
use as reference).

### Define a metric type

Create a new module under `src/metrics/`, and define your type:

    struct MyCustomMetric;

You can define any fields in that struct; you'd usually use them to keep track
of counters between individual rendering runs (in order to calculate rates, for
example).

### Implement the `Metric` trait

Implement the `Metric` trait for your type.

    pub trait Metric {
            fn init(&mut self, st: &mut MetricState, initarg: Option<String>);
            fn render(&mut self, st: &mut MetricState) -> RenderResult;
    }

The `init()` method takes a `MetricState` object, and an optional argument. The
argument is a user-supplied string from the command line invocation (see below,
"Register your metric"). `MetricState` is a quite simple type (at the moment):

    impl MetricState {
        // Obtain current timestamp.
        pub fn now() -> i64;
        // Time (in milliseconds since Unix epoch) of last call.
        pub last_called: i64;
    }

Every time your metric is asked to `render()`, it is given the same
`MetricState` object (and of course the very same underlying Metric);
`last_called` is set to the timestamp of the previous invocation (so you can
compute a rate, for example); it has millisecond resolution and is set
to 0 on the first invocation.

Typically you will set some constant configuration parameters on your metric
object when your `init()` method is invoked, and use them later to determine
how exactly you'll render the metric.

At the end of your `render()` implementation, you return a `RenderResult`:

    // use framework::*;
    fn RenderResult::new(text: String, color: Color) -> RenderResult

This is the actual output that will appear.

Finally, export a factory function that creates an instance of your metric
(this way, you can keep the rest of your module private):

    pub fn make_your_metric() -> Box<Metric> { Box::new(MyCustomMetric) }

### Register your new metric

In `src/main.rs`, you need to register your metric inside the
`register_metrics()` function near the end of the file:

    use metric::your_metric;
    registry.register_metric(
            "your_metric",  // metric name; used as command line flag
            "A metric that shows how frobnicated your foos are",  // Short description
            "format string",  // config parameter description
            your_metric::make_your_metric());  // An instance of your metric, of type Box<Metric>

This will add a command line flag that can be specified by users to activate
your metric, i.e., make it show up. There will also be a snippet of
documentation that is displayed when the user calls for `--help`:

    --your_metric [format string] A metric that shows how frobnicated your foos are

If a user invokes `myi3stat` like this, your new metric will be displayed:

    myi3stat --your_metric "%f %f %d"

### Use it

Edit your i3 configuration, e.g. at `.config/i3/config` or `.i3/config`; add
(or change) the following configuration snippet at the end of the file:

    bar {
        status_command /path/to/myi3stat --interval 5 --your_metric "%f %f"
        tray_output primary
    }

Voilà!

If you have several metrics, the default output is in alphabetical order; you can use the
`--ordering` flag to change this behavior:

    myi3stat --ordering your_metric,clock --your_metric "%f %f" --clock

## Advanced: How to add your renderer

You can also customize the rendering part. By implementing the `Renderer` trait (defined in `src/render.rs`) and
registering it within `register_renderers()` in `src/main.rs`, you can define arbitrary -- usually text-based -- renderers.

The currently available renderers are

* i3status: This renders the metrics in a format suitable for `i3bar` to display them.
* plain: This renders metrics in a way similar to `dstat`, using ANSI colors on stdout.

The renderer to be used is selected using the `--renderer` option:

    myi3stat --interval 1000 --renderer plain [...]



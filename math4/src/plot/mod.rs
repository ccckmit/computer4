pub mod options;
pub mod device;
pub mod rplot;

pub use options::{BoxPlotOptions, HistOptions, PlotOptions, PlotType};
pub use device::{dev_off, png, svg, clear, DeviceType};
pub use rplot::{plot, hist, boxplot, qqnorm};
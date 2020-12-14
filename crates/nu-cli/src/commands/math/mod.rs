pub mod abs;
pub mod avg;
pub mod ceil;
pub mod command;
pub mod eval;
pub mod floor;
pub mod max;
pub mod median;
pub mod min;
pub mod mode;
pub mod product;
pub mod round;
pub mod stddev;
pub mod sum;
pub mod variance;

mod reducers;
mod utils;

pub use abs::SubCommand as MathAbs;
pub use avg::SubCommand as MathAverage;
pub use ceil::SubCommand as MathCeil;
pub use command::Command as Math;
pub use eval::SubCommand as MathEval;
pub use floor::SubCommand as MathFloor;
pub use max::SubCommand as MathMaximum;
pub use median::SubCommand as MathMedian;
pub use min::SubCommand as MathMinimum;
pub use mode::SubCommand as MathMode;
pub use product::SubCommand as MathProduct;
pub use round::SubCommand as MathRound;
pub use stddev::SubCommand as MathStddev;
pub use sum::SubCommand as MathSummation;
pub use variance::SubCommand as MathVariance;

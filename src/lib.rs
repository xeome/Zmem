mod memory;
mod process;
mod utils;

use utils::*;

pub use memory::*;
pub use process::*;

type AnyError = Box<dyn std::error::Error + Send + Sync>;
type Result<T = (), E = AnyError> = std::result::Result<T, E>;

pub mod pool;
pub mod uci;

pub use pool::{AnalysisJob, AnalysisResult, EnginePool, PoolProgress};
pub use uci::{discover_engine_path, UciEngine, UciEval};

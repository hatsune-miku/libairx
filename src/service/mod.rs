pub mod airx_service;
pub mod discovery_service;
pub mod data_service;
pub mod context;
pub mod handler;

pub type ShouldInterruptFunctionType = Box<dyn (Fn() -> bool) + Send + Sync>;

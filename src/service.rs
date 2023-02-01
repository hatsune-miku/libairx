pub mod airx_service;
pub mod text_service;
pub mod discovery_service;

pub type ShouldInterruptType = Box<dyn (Fn() -> bool) + Send + Sync>;

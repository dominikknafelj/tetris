// Core sound functionality
mod manager;
pub use manager::GameSounds;

// Re-export mock sound implementations for tests
#[cfg(test)]
pub mod mocks;

// Include tests in this module
#[cfg(test)]
pub mod tests; 
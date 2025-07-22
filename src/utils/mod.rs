/*!
 * Utilities Module - Refactored with Simplified Implementations
 *
 * Modular utility functions organized by functionality with
 * simplified implementations for better maintainability.
 */

pub mod cache;
pub mod math;
pub mod performance;
pub mod price;
pub mod simplified;
pub mod validation;

// Re-export commonly used utilities
pub use cache::*;
pub use math::*;
pub use performance::*;
pub use price::*;
pub use simplified::*;
pub use validation::*;

// Additional utility exports for compatibility
pub use simplified::{
    SimplifiedMath as MathOps, SimplifiedPrice as PriceUtils,
    SimplifiedValidation as ValidationUtils,
};

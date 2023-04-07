#![no_std]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub use rust_patch_derive::Patch;

/// A struct other structs can be patched with
pub trait Patch<Target> {
    /// Apply self to target
    fn apply(self, target: Target) -> Target;
}

pub mod apply {
    pub fn direct<L, R>(_l: L, r: R) -> R {
        R
    }
}

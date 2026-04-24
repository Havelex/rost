//! Kernel console subsystem.
//!
//! Provides the pixel-level framebuffer renderer ([`framebuffer`]), the PSF
//! font loader ([`font`]), the stateful [`writer::Console`], and the
//! [`print!`] / [`println!`] / [`cls!`] / [`draw_cursor!`] / [`erase_cursor!`]
//! macros.

mod font;
/// Framebuffer pixel access primitives used by the text console.
pub(crate) mod framebuffer;
#[macro_use]
/// User-facing console printing and control macros.
pub mod macros;
/// Stateful console writer implementation.
pub(crate) mod writer;

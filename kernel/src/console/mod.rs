//! Kernel console subsystem.
//!
//! Provides the pixel-level framebuffer renderer ([`framebuffer`]), the PSF
//! font loader ([`font`]), the stateful [`writer::Console`], and the
//! [`print!`] / [`println!`] / [`cls!`] / [`draw_cursor!`] / [`erase_cursor!`]
//! macros.

mod font;
pub(crate) mod framebuffer;
#[macro_use]
pub mod macros;
pub(crate) mod writer;

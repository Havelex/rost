//! Limine bootloader request statics.
//!
//! Each `#[used]` static placed in the `.limine_reqs` link section is detected
//! by the Limine bootloader at load time.  The bootloader fills in a response
//! pointer before transferring control to the kernel entry point.

use limine::{
    BaseRevision, RequestsEndMarker, RequestsStartMarker,
    request::{ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemmapRequest, StackSizeRequest},
};

#[used]
#[unsafe(link_section = ".limine_req_start")]
static REQUEST_START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(6);

/// Limine memory-map request. Populated with the physical memory layout before kernel entry.
#[used]
#[unsafe(link_section = ".limine_reqs")]
pub static MEM_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

/// Limine framebuffer request. Populated with a framebuffer descriptor before kernel entry.
#[used]
#[unsafe(link_section = ".limine_reqs")]
pub static FB_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// Limine HHDM (Higher-Half Direct Map) request. Provides the virtual offset applied to all physical addresses.
#[used]
#[unsafe(link_section = ".limine_reqs")]
pub static HDDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Limine stack-size request. Requests an 32 KiB boot stack from the bootloader.
#[used]
#[unsafe(link_section = ".limine_reqs")]
static STACK_REQUEST: StackSizeRequest = StackSizeRequest::new(0x8000);

/// Limine kernel-address request. Provides the physical and virtual base addresses of the kernel image.
#[used]
#[unsafe(link_section = ".limine_reqs")]
pub static KERNEL_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".limine_req_end")]
static REQUEST_END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

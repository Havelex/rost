use limine::{
    BaseRevision, RequestsEndMarker, RequestsStartMarker,
    request::{FramebufferRequest, HhdmRequest, MemmapRequest, StackSizeRequest},
};

#[used]
#[unsafe(link_section = ".limine_req_start")]
static REQUEST_START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(6);

#[used]
#[unsafe(link_section = ".limine_reqs")]
pub static MEM_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
pub static FB_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
pub static HDDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".limine_reqs")]
static STACK_REQUEST: StackSizeRequest = StackSizeRequest::new(0x8000);

#[used]
#[unsafe(link_section = ".limine_req_end")]
static REQUEST_END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

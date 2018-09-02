use hal9000::mem::{
    page::{Page, TableUpdate}
};

#[must_use = "the TLB must be flushed to commit page table updates"]
pub struct FlushTlb<P: Page> {
    pub(crate) page: P,
}

impl<P: Page> TableUpdate for FlushTlb<P> {
    type Item = ();
    unsafe fn commit(self) -> Self::Item {
         asm!( "invlpg [$0]"
             :
             : "r" (self.page)
             : "memory"
             : "intel", "volatile" );
        // TODO: consider returning the page?
    }
}

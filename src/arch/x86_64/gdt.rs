use lazy_static::lazy_static;
use x86_64::instructions::segmentation::CS;
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
static STACK_SIZE: usize = 4096 * 5;
// x86_64 is byte-addressable, so each `u8` represents one byte of stack storage.
static mut DOUBLE_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    // The TSS stores stack information used during exception handling.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        #[allow(unused_unsafe)]
        let stack_start = VirtAddr::from_ptr(unsafe { &raw const DOUBLE_FAULT_STACK});
        let stack_end = stack_start + STACK_SIZE as u64;

        // The x86_64 stack grows downward, so the initial stack pointer must start at stack_end.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;
        tss
    };
    // The GDT is needed so the CPU can use the TSS, including the dedicated stack for double faults.
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // The code selector is loaded into the CS (Code Segment) register.
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

pub fn init_gdt() {
    use x86_64::instructions::segmentation::Segment;

    // Load the GDT pointer into the GDTR register.
    GDT.0.load();

    unsafe {
        // Load the code selector into the CPU's CS register.
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

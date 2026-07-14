use lazy_static::lazy_static;
use x86_64::instructions::segmentation::CS;
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

// IST (Interrupt Stack Table) index used for the double fault handler.
// The IST is a table of up to 7 alternative stack pointers stored inside the TSS.
// Each IDT entry can specify an IST index; when the CPU delivers that exception it
// automatically switches RSP to the corresponding IST stack before calling the handler.
// Index 0 is the first slot; any value 0–6 would work as long as it matches the IDT entry.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Physical storage for the double fault handler's dedicated stack (20 KiB).
// Declared as a plain byte array in the BSS so no allocator is required at
// this early stage of initialization.
static STACK_SIZE: usize = 4096 * 5;
// x86_64 is byte-addressable, so each `u8` represents one byte of stack storage.
static mut DOUBLE_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    // TSS (Task State Segment) is a CPU-level structure that x86_64 requires to be
    // present even though the OS does not use hardware task switching.
    // Its main job in a modern 64-bit kernel is to hold the IST (Interrupt Stack
    // Table): up to 7 virtual addresses that the CPU can automatically switch RSP
    // to when delivering certain exceptions.
    //
    // Why a dedicated stack for double faults?
    // A double fault fires when the CPU cannot invoke the handler for a *previous*
    // exception. The most common cause is a kernel stack overflow: if RSP points
    // into unmapped memory, the CPU cannot push the exception frame and triggers a
    // double fault. But if the double fault handler itself tries to use the same
    // overflowed stack, the CPU cannot deliver *that* either, and the machine
    // triple-faults and resets — with no error message at all.
    // Giving the double fault handler its own IST stack breaks that cycle.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        #[allow(unused_unsafe)]
        let stack_start = VirtAddr::from_ptr(unsafe { &raw const DOUBLE_FAULT_STACK});
        let stack_end = stack_start + STACK_SIZE as u64;

        // The x86_64 stack grows downward, so the initial stack pointer must point
        // to the *top* (highest address) of the allocated region.
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

    // `lgdt` instruction: writes the GDT's base address and size (limit) into
    // the CPU's GDTR (Global Descriptor Table Register). After this call the
    // CPU knows *where* the GDT lives in memory, but the segment registers
    // (CS, SS, TR, …) still point at whatever the bootloader set up.
    GDT.0.load();

    unsafe {
        // CS (Code Segment) register holds a *selector* — an index into the
        // GDT — that tells the CPU which descriptor governs instruction fetches.
        // The bootloader's CS selector pointed at its own GDT entry; now that we
        // have loaded our own GDT we must update CS to our kernel code descriptor,
        // otherwise the CPU would be reading descriptor data from an unmapped or
        // wrong entry.
        //
        // On x86_64, CS cannot be changed with a plain `mov`; a far jump or
        // `retfq` is required. The x86_64 crate handles that with inline asm,
        // which is why the call is inside an `unsafe` block.
        CS::set_reg(GDT.1.code_selector);

        // `ltr` instruction: loads the TSS selector into the TR (Task Register).
        // The CPU uses the TR to locate the TSS (Task State Segment) at runtime.
        // In particular, when a double fault fires the CPU reads
        // `TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX]` to find the
        // dedicated stack to switch to before invoking the handler. Without
        // loading the TR here, the IST entry in the TSS is effectively invisible
        // to the CPU and the double fault handler would run on the (possibly
        // corrupted or exhausted) normal kernel stack.
        load_tss(GDT.1.tss_selector);
    }
}

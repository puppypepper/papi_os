# Network Stack Roadmap for `papi_os`

## Current Starting Point

Right now `papi_os` is still in the earliest kernel stage:

- It boots into `_start`.
- It can write text to the VGA text buffer.
- It does not yet have robust exception handling, interrupts, memory management, device discovery, or driver infrastructure.

That means the shortest realistic path to "talk to the host OS over the network" is not to jump directly into TCP/IP. The right path is to build the kernel foundations first, then add one network device driver, then add only the protocol layers needed for a first successful end-to-end exchange.

## Recommended Strategy

Use this target stack:

- Platform: `x86_64`
- Emulator: `QEMU`
- First NIC target: `virtio-net` if you want a modern and practical path, or `e1000` if you want a simpler, better-documented teaching path
- First host communication goal: `ICMP ping` or a tiny `UDP` exchange with the host

If the goal is learning and steady progress, a good order is:

1. Make the kernel debuggable.
2. Make the kernel structurally correct.
3. Add memory and interrupt infrastructure.
4. Add PCI and one NIC driver.
5. Add Ethernet, ARP, IPv4, and one very small transport/demo protocol.
6. Only then consider TCP.

## Phase 1: Make Development Reliable

Before adding features, make the kernel easy to observe and debug.

### Goals

- Add serial output through `COM1` so logs are visible in QEMU even when VGA output breaks.
- Improve panic handling so panics print useful information.
- Add a `hlt` loop instead of a pure busy loop.
- Add a minimal test workflow for boot checks.
- Make the VGA writer usable enough for multi-line text output.

### Why this matters

Networking work is difficult without logs. Serial logging will save a large amount of time once you start handling interrupts, PCI config space, and packet parsing.

### Deliverables

- `println!`-style output to VGA and serial.
- Panic messages printed to serial.
- `cargo run` or an equivalent one-command QEMU workflow.
- A fixed `new_line()` implementation in the VGA writer.

## Phase 2: CPU and Kernel Bring-Up Basics

Build the minimum kernel runtime needed before touching devices.

### Goals

- Add a GDT and IDT.
- Handle CPU exceptions such as breakpoint and page fault.
- Initialize interrupts cleanly.
- Add timer interrupts.
- Add keyboard input only if you want interactive debugging.

### Why this matters

A NIC driver depends on interrupts and safe fault handling. If a bad MMIO access or DMA-related bug crashes the kernel without diagnostics, progress becomes very slow.

### Deliverables

- Working exception handlers.
- PIC or APIC initialization.
- Timer interrupt logs.
- A stable idle loop using `hlt`.

## Phase 3: Memory Management Foundations

You do not need a full Unix-like memory system yet, but you do need controlled allocation.

### Goals

- Understand and initialize paging state.
- Add a physical frame allocator.
- Add a small kernel heap allocator.
- Introduce clear rules for `unsafe` memory access.

### Why this matters

Network drivers need buffers, descriptor rings, and sometimes DMA-friendly memory. Without a basic allocator and memory model, driver work becomes ad hoc and fragile.

### Deliverables

- Heap-backed dynamic allocation.
- A frame allocator for physical pages.
- Early documentation for memory ownership and address translation rules.

## Phase 4: Kernel Structure and Internal Interfaces

Before drivers appear, shape the code so they have somewhere reasonable to live.

### Goals

- Split the kernel into modules such as `arch`, `memory`, `interrupts`, `drivers`, and `net`.
- Add logging levels or at least consistent log prefixes.
- Define simple abstractions for device initialization and packet buffers.
- Decide early whether you want a "small pragmatic kernel" or a more abstract microkernel-like design.

### Why this matters

A network stack becomes messy quickly if packet parsing, buffer ownership, hardware access, and protocol logic all live in one place.

### Deliverables

- Basic module layout.
- Clear ownership boundaries between architecture code, drivers, and protocol code.
- A packet buffer type that can evolve later.

## Phase 5: Device Discovery

Once the kernel basics are stable, teach the kernel how to find hardware.

### Goals

- Add PCI configuration space access.
- Enumerate PCI devices and print them to serial.
- Identify the virtual NIC exposed by QEMU.
- Choose one NIC to support first and ignore all others.

### Recommendation

Pick one of these:

- `virtio-net`: better long-term direction if you want a modern virtualized path.
- `e1000`: often easier for a first educational driver because there are many examples and writeups.

Do not try to support multiple NICs early.

### Deliverables

- PCI scan output visible in serial logs.
- Confirmed vendor/device ID for the first NIC target.
- A documented QEMU launch command that consistently exposes that NIC.

## Phase 6: First NIC Driver

This is the point where the OS starts becoming a system instead of a boot demo.

### Goals

- Initialize the NIC.
- Read the NIC MAC address.
- Set up TX and RX descriptor rings.
- Receive interrupts or poll in a controlled way for initial bring-up.
- Send a raw Ethernet frame.
- Receive a raw Ethernet frame.

### Implementation advice

- Start with polling if interrupts slow you down.
- Switch to interrupts after transmit and receive paths are working.
- Log every step of initialization and every error case.

### Deliverables

- Driver init routine completes successfully.
- MAC address printed to serial.
- Transmit path works for a handcrafted Ethernet frame.
- Receive path can dump frame metadata and bytes.

## Phase 7: Link Layer Support

Now turn the driver into a usable network interface.

### Goals

- Define an Ethernet frame type.
- Add MAC address handling.
- Add ARP request and reply parsing.
- Resolve the host MAC address through ARP.

### Why this matters

If your guest can resolve the host MAC and exchange Ethernet frames, you have crossed the boundary from "device driver" to "networking".

### Deliverables

- Outbound ARP request.
- Inbound ARP reply.
- ARP cache with at least one simple entry.

## Phase 8: Minimal IPv4

Do the minimum needed for host communication.

### Goals

- Add an IPv4 header parser and serializer.
- Assign the guest a static IPv4 address at first.
- Parse inbound IPv4 packets addressed to the guest.
- Validate header length and checksum.

### Keep scope narrow

At this stage, skip fragmentation, routing complexity, and advanced configuration. Static addressing is fine.

### Deliverables

- Outbound IPv4 packet creation.
- Inbound IPv4 packet validation.
- Clear separation between Ethernet and IPv4 layers.

## Phase 9: First End-to-End Host Communication

Choose the simplest proof that the guest and host can actually exchange useful traffic.

### Option A: ICMP Echo

Implement enough ICMP to reply to ping or send ping.

This is a very good first proof because:

- It is simple.
- It validates Ethernet, ARP, IPv4, and checksums.
- It is easy to inspect with `tcpdump` or Wireshark on the host.

### Option B: UDP

Implement minimal UDP send and receive.

This is a very good first application path because:

- It is still simple.
- You can write a tiny host-side UDP server quickly.
- It gives you a foundation for later protocols.

### Recommended order

1. ARP
2. IPv4
3. ICMP echo
4. UDP
5. TCP only much later

### Deliverables

- Host can `ping` the guest, or the guest can `ping` the host.
- Or, guest can send a UDP packet and the host can reply.
- Packet traces confirm correctness.

## Phase 10: Host Integration Setup

Be explicit about how the guest will reach the host.

### Practical setups

- QEMU user-mode networking: easier to start, but sometimes awkward for direct host-guest experiments.
- TAP or bridge networking: more setup, but better for realistic packet flow and direct testing.

### Recommendation

For serious driver and protocol debugging, move to TAP or bridge networking once the NIC driver starts working.

### Deliverables

- A documented local networking setup.
- A repeatable host-side test command such as `ping`, `nc`, or a tiny Python/Rust UDP tool.
- Packet capture instructions for the host.

## Phase 11: Stabilize the Stack

Once packets are moving, improve correctness before adding more protocols.

### Goals

- Add checksums everywhere they are required.
- Handle malformed packets safely.
- Avoid buffer overruns and invalid pointer use.
- Add timeouts and retry logic for ARP.
- Add basic statistics counters.

### Deliverables

- No crash on malformed frames.
- Clear counters for TX, RX, drops, checksum failures, and ARP misses.
- Better logs for debugging packet flow.

## Phase 12: Optional Next Steps

After the first successful host communication, you can choose depth based on your goals.

### If your goal is systems learning

- Add a cleaner driver model.
- Support interrupts fully.
- Add multiple NICs or multiple queues.
- Add a small shell or monitor.

### If your goal is application communication

- Improve UDP.
- Add DHCP instead of static addressing.
- Add DNS if you eventually want outbound name resolution.
- Add a tiny application protocol between guest and host.

### If your goal is a full network stack

- Add TCP.
- Add retransmission, windows, and connection state management.
- Add routing and richer socket-like APIs.

TCP is a major project. It should not be an early milestone.

## Suggested Milestone Order

This is the version I would actually recommend following:

1. Serial logging, panic output, and stable boot workflow.
2. GDT, IDT, exceptions, timer interrupt, idle loop.
3. Paging understanding, frame allocator, heap allocator.
4. Kernel module cleanup and internal interfaces.
5. PCI scan and NIC identification.
6. First NIC driver with raw frame TX/RX.
7. Ethernet and ARP.
8. IPv4.
9. ICMP echo.
10. UDP request and reply with the host.
11. Reliability and debugging tools.
12. TCP only after everything above is stable.

## What Not to Do Early

Avoid these traps:

- Do not start with TCP.
- Do not support multiple NICs at first.
- Do not mix protocol design and driver debugging in the same step if you can separate them.
- Do not rely on VGA text only for diagnostics.
- Do not add too much abstraction before you have one working driver and one working protocol path.

## Concrete Near-Term Plan for This Repository

Given the current codebase, the next few steps should be very small and practical:

1. Finish the VGA writer so scrolling and newlines work.
2. Add serial output and route panic messages there.
3. Add `hlt` in the idle loop.
4. Add GDT, IDT, and exception handlers.
5. Add QEMU-friendly debugging and test support.
6. Add paging and heap allocation.
7. Add PCI enumeration.
8. Choose one NIC and write the first driver.
9. Send and receive raw Ethernet frames.
10. Implement ARP and ICMP or UDP for the first host exchange.

## Definition of Success

The first real success milestone should be narrow and testable:

> `papi_os` boots in QEMU, initializes one NIC, obtains or uses a known IP configuration, and successfully exchanges packets with the host OS, with packet flow visible in serial logs and host-side packet capture.

That is the point where you can honestly say the OS has moved beyond "it prints text" and into "it communicates with another machine".

// A no-op `Waker` that never actually wakes anything.
//
// This is the pass-1 waker, paired with `SimpleExecutor`. It exists on
// purpose, not as dead-code drift: `SimpleExecutor` re-polls every
// not-yet-finished task on every loop iteration regardless of whether it was
// woken, so the waker it hands to `poll()` never needs to do anything. That
// makes `SimpleExecutor` correct but busy-loops at 100% CPU forever, which is
// why it is kept only as a simple, always-on logging executor (see its own
// doc comment) and not used for the kernel's real scheduling path -
// `BTreeMapExecutor` + `TaskWaker` (in `task::waker::task_waker`) replace
// this with wakers that can actually signal "poll me again", so that path
// can `hlt` between wakeups instead of spinning.
#![allow(dead_code)]

use core::task::{RawWaker, RawWakerVTable, Waker};

// `RawWakerVTable::new` takes plain `fn` pointers, not closures - a closure
// isn't guaranteed to coerce to the bare `unsafe fn(*const ())` shape a
// vtable slot needs. Three of the four slots below are the same no-op body;
// only `clone` needs a real implementation, and even that is trivial here
// because this waker carries no data to duplicate.
static VTABLE: RawWakerVTable =
    RawWakerVTable::new(dummy_raw_waker, wake, wake_by_ref, core::mem::drop);

// Doubles as both "build the initial `RawWaker`" and the vtable's `clone`
// slot: `clone`'s contract is "produce another raw waker just like this
// one", and since there is no per-waker state (the data pointer is always
// null), that is exactly what calling this function again does.
unsafe fn dummy_raw_waker(_: *const ()) -> RawWaker {
    // `core::ptr::null()` instead of e.g. `&()` - there is no real data
    // behind this waker, and a null pointer makes that honest instead of
    // handing out a technically-dangling reference to a dropped temporary.
    RawWaker::new(core::ptr::null(), &VTABLE)
}

// Safe wrapper around the raw waker above. `Waker::from_raw` is `unsafe`
// because it trusts the vtable's contract; that trust is upheld once, here,
// so every caller of `dummy_waker()` gets a safe, ordinary `Waker` back.
pub fn dummy_waker() -> Waker {
    let raw_waker = unsafe { dummy_raw_waker(core::ptr::null()) };
    unsafe { Waker::from_raw(raw_waker) }
}

// `wake`/`wake_by_ref` do nothing: see the module comment above for why that
// is correct for `SimpleExecutor` specifically, and not a shortcut taken by
// mistake.
unsafe fn wake(_: *const ()) {}
unsafe fn wake_by_ref(_: *const ()) {}

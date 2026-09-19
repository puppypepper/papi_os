use core::task::{RawWaker, RawWakerVTable, Waker};

static VTABLE: RawWakerVTable = RawWakerVTable::new(dummy_raw_waker, wake, wake_by_ref, core::mem::drop);

unsafe fn dummy_raw_waker(_: *const ()) -> RawWaker {
    RawWaker::new(core::ptr::null(), &VTABLE)
}

pub fn dummy_waker() -> Waker {
    let raw_waker = unsafe { dummy_raw_waker(core::ptr::null()) };
    unsafe { Waker::from_raw(raw_waker) }
}

unsafe fn wake(_: *const ()) {}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}

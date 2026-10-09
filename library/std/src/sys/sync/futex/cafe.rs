use crate::cell::UnsafeCell;
use crate::ptr;
use crate::sync::atomic::Atomic;
use crate::sync::atomic::Ordering::Relaxed;
use crate::sys::pal::abi;
use crate::time::Duration;

pub type Futex = Atomic<Primitive>;
pub type Primitive = u32;

pub type SmallFutex = Atomic<SmallPrimitive>;
pub type SmallPrimitive = u32;

/// A thread blocked in `futex_wait`. Lives on that thread's stack and is only
/// accessed while holding its bucket's lock.
struct Waiter {
    futex: *const Atomic<u32>,
    event: abi::OSEvent,
    next: *mut Waiter,
    woken: bool,
}

struct Bucket {
    lock: UnsafeCell<abi::OSFastMutex>,
    waiters: UnsafeCell<*mut Waiter>,
}

// SAFETY: `waiters` is only accessed while holding `lock`.
unsafe impl Sync for Bucket {}

struct BucketGuard(&'static Bucket);

impl BucketGuard {
    fn lock(bucket: &'static Bucket) -> BucketGuard {
        unsafe { abi::OSFastMutex_Lock(bucket.lock.get()) };
        BucketGuard(bucket)
    }

    fn waiters(&mut self) -> *mut *mut Waiter {
        self.0.waiters.get()
    }
}

impl Drop for BucketGuard {
    fn drop(&mut self) {
        unsafe { abi::OSFastMutex_Unlock(self.0.lock.get()) }
    }
}

const BUCKET_COUNT: usize = 64;

static BUCKETS: [Bucket; BUCKET_COUNT] = [const {
    Bucket {
        lock: UnsafeCell::new(abi::OSFastMutex::ZEROED),
        waiters: UnsafeCell::new(ptr::null_mut()),
    }
}; BUCKET_COUNT];

fn bucket(futex: &Atomic<u32>) -> &'static Bucket {
    &BUCKETS[(ptr::from_ref(futex).addr() >> 2) % BUCKET_COUNT]
}

pub fn futex_wait(futex: &Atomic<u32>, expected: u32, timeout: Option<Duration>) -> bool {
    let bucket = bucket(futex);
    let mut waiter =
        Waiter { futex, event: abi::OSEvent::ZEROED, next: ptr::null_mut(), woken: false };
    let waiter = &raw mut waiter;
    let event = unsafe { &raw mut (*waiter).event };
    unsafe { abi::OSInitEvent(event, 0, abi::OS_EVENT_MANUAL) };

    {
        let mut guard = BucketGuard::lock(bucket);
        if futex.load(Relaxed) != expected {
            return true;
        }
        // Append, so that waiters are woken in FIFO order.
        let mut tail = guard.waiters();
        unsafe {
            while !(*tail).is_null() {
                tail = &raw mut (**tail).next;
            }
            *tail = waiter;
        }
    }

    match timeout {
        None => unsafe { abi::OSWaitEvent(event) },
        Some(dur) => {
            let nanos = i64::try_from(dur.as_nanos()).unwrap_or(i64::MAX);
            unsafe { abi::OSWaitEventWithTimeout(event, nanos) };
        }
    }

    // Wakers signal while holding the lock, so taking it here also guarantees
    // that no waker still touches `waiter` once this function returns.
    let mut guard = BucketGuard::lock(bucket);
    let woken = unsafe { (*waiter).woken };
    if !woken {
        // Timed out: remove ourselves from the list.
        let mut cursor = guard.waiters();
        unsafe {
            while *cursor != waiter {
                cursor = &raw mut (**cursor).next;
            }
            *cursor = (*waiter).next;
        }
    }
    woken
}

#[inline]
pub fn futex_wake(futex: &Atomic<u32>) -> bool {
    wake(futex, false)
}

#[inline]
pub fn futex_wake_all(futex: &Atomic<u32>) {
    wake(futex, true);
}

fn wake(futex: &Atomic<u32>, all: bool) -> bool {
    let target = ptr::from_ref(futex);
    let mut guard = BucketGuard::lock(bucket(futex));
    let mut cursor = guard.waiters();
    let mut woke = false;
    unsafe {
        while !(*cursor).is_null() {
            let waiter = *cursor;
            if (*waiter).futex != target {
                cursor = &raw mut (*waiter).next;
                continue;
            }
            *cursor = (*waiter).next;
            (*waiter).woken = true;
            abi::OSSignalEvent(&raw mut (*waiter).event);
            woke = true;
            if !all {
                break;
            }
        }
    }
    woke
}

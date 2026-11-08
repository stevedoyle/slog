//! A bounded, blocking, thread-safe producer/consumer queue.
//!
//! [`Queue`] is multi-producer, multi-consumer (MPMC). Share it between
//! threads with an [`Arc`](std::sync::Arc).
//!
//! # Move semantics
//!
//! `T` has no `Clone` or `Copy` bound. Every value is *moved* into the queue
//! by a push and *moved* out to exactly one consumer by a pop. When a push
//! fails (queue full or closed) the value is handed back inside the error, so
//! a value is never silently dropped or duplicated.
//!
//! # Efficiency
//!
//! * The ring buffer is allocated once, at construction, with the full
//!   capacity. Pushes and pops never reallocate.
//! * The critical section only moves a value in or out of the buffer. No user
//!   code (`Drop`, closures) runs while the lock is held.
//! * Producers wait on `not_full` and consumers on `not_empty`, so a push only
//!   wakes a consumer and a pop only wakes a producer. Each wakes one thread
//!   (`notify_one`), after the lock is released, so the woken thread does not
//!   immediately block on the mutex.
//!
//! # Shutdown
//!
//! Call [`Queue::close`] when no more values will be produced. Blocked
//! producers wake and get their value back as an error; consumers keep
//! draining the remaining values, then [`Queue::pop`] returns `None`.
//!
//! # Example
//!
//! ```
//! use std::num::NonZeroUsize;
//! use std::sync::Arc;
//! use std::thread;
//!
//! use pcq::Queue;
//!
//! let q = Arc::new(Queue::new(NonZeroUsize::new(16).unwrap()));
//!
//! let producer = {
//!     let q = Arc::clone(&q);
//!     thread::spawn(move || {
//!         for i in 0..100 {
//!             q.push(format!("job {i}")).expect("queue closed early");
//!         }
//!         q.close();
//!     })
//! };
//!
//! let mut received = 0;
//! while let Some(job) = q.pop() {
//!     assert!(job.starts_with("job "));
//!     received += 1;
//! }
//! producer.join().unwrap();
//! assert_eq!(received, 100);
//! ```

use std::collections::VecDeque;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// A bounded, blocking MPMC queue. See the [crate docs](crate) for details.
pub struct Queue<T> {
    state: Mutex<State<T>>,
    not_empty: Condvar,
    not_full: Condvar,
    capacity: usize,
}

struct State<T> {
    items: VecDeque<T>,
    closed: bool,
}

/// Returned by [`Queue::push`] when the queue is closed. Holds the value
/// that could not be pushed.
#[derive(PartialEq, Eq)]
pub struct Closed<T>(pub T);

/// Returned by [`Queue::try_push`] and [`Queue::push_timeout`]. Holds the
/// value that could not be pushed.
#[derive(PartialEq, Eq)]
pub enum PushError<T> {
    /// The queue was full (or stayed full until the timeout).
    Full(T),
    /// The queue is closed.
    Closed(T),
}

/// Returned by [`Queue::try_pop`] and [`Queue::pop_timeout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopError {
    /// The queue was empty (or stayed empty until the timeout).
    Empty,
    /// The queue is closed and fully drained.
    Closed,
}

impl<T> Queue<T> {
    /// Creates a queue that holds at most `capacity` values.
    ///
    /// The buffer is allocated up front.
    pub fn new(capacity: NonZeroUsize) -> Self {
        let capacity = capacity.get();
        Self {
            state: Mutex::new(State {
                items: VecDeque::with_capacity(capacity),
                closed: false,
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        }
    }

    /// Moves `value` into the queue, blocking while the queue is full.
    ///
    /// Returns the value inside [`Closed`] if the queue is, or becomes,
    /// closed before there is room.
    pub fn push(&self, value: T) -> Result<(), Closed<T>> {
        let mut state = self.lock();
        while !state.closed && state.items.len() == self.capacity {
            state = self.wait(&self.not_full, state);
        }
        if state.closed {
            return Err(Closed(value));
        }
        self.enqueue(state, value);
        Ok(())
    }

    /// Moves `value` into the queue without blocking.
    pub fn try_push(&self, value: T) -> Result<(), PushError<T>> {
        let state = self.lock();
        if state.closed {
            return Err(PushError::Closed(value));
        }
        if state.items.len() == self.capacity {
            return Err(PushError::Full(value));
        }
        self.enqueue(state, value);
        Ok(())
    }

    /// Moves `value` into the queue, blocking for at most `timeout` while
    /// the queue is full.
    pub fn push_timeout(&self, value: T, timeout: Duration) -> Result<(), PushError<T>> {
        let deadline = Instant::now().checked_add(timeout);
        let mut state = self.lock();
        while !state.closed && state.items.len() == self.capacity {
            match self.wait_until(&self.not_full, state, deadline) {
                Some(s) => state = s,
                None => return Err(PushError::Full(value)),
            }
        }
        if state.closed {
            return Err(PushError::Closed(value));
        }
        self.enqueue(state, value);
        Ok(())
    }

    /// Moves the oldest value out of the queue, blocking while it is empty.
    ///
    /// Returns `None` once the queue is closed and every value has been
    /// popped.
    pub fn pop(&self) -> Option<T> {
        let mut state = self.lock();
        loop {
            if let Some(value) = state.items.pop_front() {
                self.dequeued(state);
                return Some(value);
            }
            if state.closed {
                return None;
            }
            state = self.wait(&self.not_empty, state);
        }
    }

    /// Moves the oldest value out of the queue without blocking.
    pub fn try_pop(&self) -> Result<T, PopError> {
        let mut state = self.lock();
        match state.items.pop_front() {
            Some(value) => {
                self.dequeued(state);
                Ok(value)
            }
            None if state.closed => Err(PopError::Closed),
            None => Err(PopError::Empty),
        }
    }

    /// Moves the oldest value out of the queue, blocking for at most
    /// `timeout` while it is empty.
    pub fn pop_timeout(&self, timeout: Duration) -> Result<T, PopError> {
        let deadline = Instant::now().checked_add(timeout);
        let mut state = self.lock();
        loop {
            if let Some(value) = state.items.pop_front() {
                self.dequeued(state);
                return Ok(value);
            }
            if state.closed {
                return Err(PopError::Closed);
            }
            match self.wait_until(&self.not_empty, state, deadline) {
                Some(s) => state = s,
                None => return Err(PopError::Empty),
            }
        }
    }

    /// Closes the queue. Idempotent.
    ///
    /// Later pushes fail and return their value. Values already queued can
    /// still be popped. Every blocked producer and consumer is woken.
    pub fn close(&self) {
        let mut state = self.lock();
        if state.closed {
            return;
        }
        state.closed = true;
        drop(state);
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }

    /// Returns `true` if [`close`](Self::close) has been called.
    pub fn is_closed(&self) -> bool {
        self.lock().closed
    }

    /// Returns the number of queued values. Other threads may change it
    /// immediately, so use it only as a hint.
    pub fn len(&self) -> usize {
        self.lock().items.len()
    }

    /// Returns `true` if no values are queued. A hint, like [`len`](Self::len).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the maximum number of values the queue holds.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Pushes onto a non-full, open queue, releases the lock and wakes one
    /// consumer.
    fn enqueue(&self, mut state: MutexGuard<'_, State<T>>, value: T) {
        state.items.push_back(value);
        drop(state);
        self.not_empty.notify_one();
    }

    /// Releases the lock after a pop and wakes one producer.
    fn dequeued(&self, state: MutexGuard<'_, State<T>>) {
        drop(state);
        self.not_full.notify_one();
    }

    // Poisoning is ignored on purpose: no user code runs while the lock is
    // held, and `VecDeque` never reallocates (capacity is reserved up front),
    // so a panic elsewhere cannot leave `State` inconsistent.
    fn lock(&self) -> MutexGuard<'_, State<T>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn wait<'a>(
        &self,
        cv: &Condvar,
        guard: MutexGuard<'a, State<T>>,
    ) -> MutexGuard<'a, State<T>> {
        cv.wait(guard).unwrap_or_else(PoisonError::into_inner)
    }

    /// Waits until notified or until `deadline`. Returns `None` when the
    /// deadline has passed. A `None` deadline (overflowed timeout) waits
    /// forever.
    fn wait_until<'a>(
        &self,
        cv: &Condvar,
        guard: MutexGuard<'a, State<T>>,
        deadline: Option<Instant>,
    ) -> Option<MutexGuard<'a, State<T>>> {
        let Some(deadline) = deadline else {
            return Some(self.wait(cv, guard));
        };
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        let (guard, _) = cv
            .wait_timeout(guard, remaining)
            .unwrap_or_else(PoisonError::into_inner);
        // Spurious or timed-out wakeups are fine: the caller rechecks its
        // condition first, and the next call notices the passed deadline.
        Some(guard)
    }
}

impl<T> fmt::Debug for Queue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.lock();
        f.debug_struct("Queue")
            .field("len", &state.items.len())
            .field("capacity", &self.capacity)
            .field("closed", &state.closed)
            .finish()
    }
}

// The error types hold a `T` that may not be `Debug`/`Display`; print only
// the condition, like `std::sync::mpsc::SendError`.
impl<T> fmt::Debug for Closed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Closed(..)")
    }
}

impl<T> fmt::Display for Closed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("push on a closed queue")
    }
}

impl<T> std::error::Error for Closed<T> {}

impl<T> PushError<T> {
    /// Recovers the value that could not be pushed.
    pub fn into_inner(self) -> T {
        match self {
            PushError::Full(v) | PushError::Closed(v) => v,
        }
    }
}

impl<T> fmt::Debug for PushError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PushError::Full(_) => f.write_str("Full(..)"),
            PushError::Closed(_) => f.write_str("Closed(..)"),
        }
    }
}

impl<T> fmt::Display for PushError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PushError::Full(_) => f.write_str("push on a full queue"),
            PushError::Closed(_) => f.write_str("push on a closed queue"),
        }
    }
}

impl<T> std::error::Error for PushError<T> {}

impl fmt::Display for PopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PopError::Empty => f.write_str("pop on an empty queue"),
            PopError::Closed => f.write_str("pop on a closed, drained queue"),
        }
    }
}

impl std::error::Error for PopError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    fn queue<T>(cap: usize) -> Queue<T> {
        Queue::new(NonZeroUsize::new(cap).expect("test capacity must be non-zero"))
    }

    /// Deliberately neither `Clone` nor `Copy`: compiling these tests proves
    /// the queue only moves values.
    #[derive(Debug, PartialEq, Eq)]
    struct MoveOnly(Box<u64>);

    #[test]
    fn fifo_order() {
        let q = queue(4);
        for i in 0..4 {
            q.push(i).unwrap();
        }
        for i in 0..4 {
            assert_eq!(q.pop(), Some(i));
        }
    }

    #[test]
    fn moves_non_clone_values() {
        let q = queue(2);
        q.push(MoveOnly(Box::new(7))).unwrap();
        assert_eq!(q.pop(), Some(MoveOnly(Box::new(7))));
    }

    #[test]
    fn try_push_full_returns_value() {
        let q = queue(1);
        q.try_push(MoveOnly(Box::new(1))).unwrap();
        match q.try_push(MoveOnly(Box::new(2))) {
            Err(PushError::Full(v)) => assert_eq!(*v.0, 2),
            other => panic!("expected Full, got {other:?}"),
        }
    }

    #[test]
    fn try_pop_empty_then_closed() {
        let q: Queue<u8> = queue(1);
        assert_eq!(q.try_pop(), Err(PopError::Empty));
        q.close();
        assert_eq!(q.try_pop(), Err(PopError::Closed));
    }

    #[test]
    fn push_after_close_returns_value() {
        let q = queue(1);
        q.close();
        assert_eq!(q.push(5).unwrap_err().0, 5);
        assert_eq!(q.try_push(6), Err(PushError::Closed(6)));
    }

    #[test]
    fn close_drains_remaining_values() {
        let q = queue(3);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.close();
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn close_wakes_blocked_consumer() {
        let q: Arc<Queue<u8>> = Arc::new(queue(1));
        let c = {
            let q = Arc::clone(&q);
            thread::spawn(move || q.pop())
        };
        thread::sleep(Duration::from_millis(50));
        q.close();
        assert_eq!(c.join().unwrap(), None);
    }

    #[test]
    fn close_wakes_blocked_producer_and_returns_value() {
        let q = Arc::new(queue(1));
        q.push(1).unwrap();
        let p = {
            let q = Arc::clone(&q);
            thread::spawn(move || q.push(2))
        };
        thread::sleep(Duration::from_millis(50));
        q.close();
        assert_eq!(p.join().unwrap().unwrap_err().0, 2);
    }

    #[test]
    fn blocked_producer_resumes_after_pop() {
        let q = Arc::new(queue(1));
        q.push(1).unwrap();
        let p = {
            let q = Arc::clone(&q);
            thread::spawn(move || q.push(2))
        };
        thread::sleep(Duration::from_millis(50));
        assert_eq!(q.pop(), Some(1));
        p.join().unwrap().unwrap();
        assert_eq!(q.pop(), Some(2));
    }

    #[test]
    fn timeouts_expire() {
        let q = queue(1);
        assert_eq!(q.pop_timeout(Duration::from_millis(20)), Err(PopError::Empty));
        q.push(1).unwrap();
        assert_eq!(
            q.push_timeout(2, Duration::from_millis(20)),
            Err(PushError::Full(2))
        );
        assert_eq!(q.pop_timeout(Duration::from_millis(20)), Ok(1));
    }

    #[test]
    fn huge_timeout_does_not_overflow() {
        let q = queue(1);
        q.push(1).unwrap();
        assert_eq!(q.pop_timeout(Duration::MAX), Ok(1));
    }

    #[test]
    fn mpmc_delivers_every_value_exactly_once() {
        const PRODUCERS: usize = 4;
        const CONSUMERS: usize = 4;
        const PER_PRODUCER: usize = 25_000;

        let q = Arc::new(queue(64));
        let seen: Arc<Vec<AtomicUsize>> = Arc::new(
            (0..PRODUCERS * PER_PRODUCER)
                .map(|_| AtomicUsize::new(0))
                .collect(),
        );

        let producers: Vec<_> = (0..PRODUCERS)
            .map(|p| {
                let q = Arc::clone(&q);
                thread::spawn(move || {
                    for i in 0..PER_PRODUCER {
                        q.push(MoveOnly(Box::new((p * PER_PRODUCER + i) as u64)))
                            .unwrap();
                    }
                })
            })
            .collect();

        let consumers: Vec<_> = (0..CONSUMERS)
            .map(|_| {
                let q = Arc::clone(&q);
                let seen = Arc::clone(&seen);
                thread::spawn(move || {
                    while let Some(MoveOnly(v)) = q.pop() {
                        seen[*v as usize].fetch_add(1, Ordering::Relaxed);
                    }
                })
            })
            .collect();

        for p in producers {
            p.join().unwrap();
        }
        q.close();
        for c in consumers {
            c.join().unwrap();
        }

        assert!(seen.iter().all(|n| n.load(Ordering::Relaxed) == 1));
        assert!(q.is_empty());
    }
}

// ---------------------------------------------------------------------------------
//
//    '%%' '%% '%%'
//    %'%\% | %/%'%     Yarrow GUI Library
//        \ | /
//         \|/          https://github.com/MeadowlarkDAW/Yarrow
//          |
//
//
// MIT License Copyright (c) 2024 Billy Messenger
// https://github.com/MeadowlarkDAW/Yarrow/blob/main/LICENSE
//
// ---------------------------------------------------------------------------------

#[cfg(not(feature = "experimental_optimizations"))]
use std::cell::RefCell;
#[cfg(feature = "experimental_optimizations")]
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// Construct a single-threaded only unbounded mpsc queue.
pub(crate) fn single_thread_mpsc_queue<T>(initial_capacity: usize) -> (Sender<T>, Receiver<T>) {
    #[cfg(not(feature = "experimental_optimizations"))]
    let queue = Rc::new(RefCell::new(VecDeque::with_capacity(initial_capacity)));

    #[cfg(feature = "experimental_optimizations")]
    let queue = Rc::new(UnsafeCell::new(VecDeque::with_capacity(initial_capacity)));

    (
        Sender {
            queue: Rc::clone(&queue),
        },
        Receiver { queue },
    )
}

/// The sending end of a single-threaded only unbounded mpsc queue.
pub(crate) struct Sender<T> {
    #[cfg(not(feature = "experimental_optimizations"))]
    queue: Rc<RefCell<VecDeque<T>>>,

    #[cfg(feature = "experimental_optimizations")]
    queue: Rc<UnsafeCell<VecDeque<T>>>,
}

impl<T> Sender<T> {
    #[inline]
    pub fn send(&mut self, msg: T) {
        #[cfg(not(feature = "experimental_optimizations"))]
        RefCell::borrow_mut(&self.queue).push_back(msg);

        #[cfg(feature = "experimental_optimizations")]
        // SAFETY: This is single-threaded only, this Vec can only be borrowed in
        // the `send()` and `try_recv()` methods inside this module, and these methods
        // only borrow them for as long as the method lasts, so it isn't possible to
        // have a data race.
        //
        // That being said, I'm not 100% sure about the safety of this (there might
        // be some compiler optimization UB shenanagins), so I'm disabling this for
        // now until I get a second opinion.
        unsafe {
            let queue = &mut *UnsafeCell::get(&self.queue);
            queue.push_back(msg);
        }
    }

    #[inline]
    pub fn send_to_front(&mut self, msg: T) {
        #[cfg(not(feature = "experimental_optimizations"))]
        RefCell::borrow_mut(&self.queue).push_front(msg);

        #[cfg(feature = "experimental_optimizations")]
        // SAFETY: This is single-threaded only, this Vec can only be borrowed in
        // the `send()` and `try_recv()` methods inside this module, and these methods
        // only borrow them for as long as the method lasts, so it isn't possible to
        // have a data race.
        //
        // That being said, I'm not 100% sure about the safety of this (there might
        // be some compiler optimization UB shenanagins), so I'm disabling this for
        // now until I get a second opinion.
        unsafe {
            let queue = &mut *UnsafeCell::get(&self.queue);
            queue.push_front(msg);
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            queue: Rc::clone(&self.queue),
        }
    }
}

/// The receiving end of a single-threaded only unbounded mpsc queue.
pub(crate) struct Receiver<T> {
    #[cfg(not(feature = "experimental_optimizations"))]
    queue: Rc<RefCell<VecDeque<T>>>,

    #[cfg(feature = "experimental_optimizations")]
    queue: Rc<UnsafeCell<VecDeque<T>>>,
}

impl<T> Receiver<T> {
    #[inline]
    pub fn try_recv(&mut self) -> Option<T> {
        #[cfg(not(feature = "experimental_optimizations"))]
        return RefCell::borrow_mut(&self.queue).pop_front();

        #[cfg(feature = "experimental_optimizations")]
        // SAFETY: This is single-threaded only, this Vec can only be borrowed in
        // the `send()` and `try_recv()` methods inside this module, and these methods
        // only borrow them for as long as the method lasts, so it isn't possible to
        // have a data race.
        //
        // That being said, I'm not 100% sure about the safety of this (there might
        // be some compiler optimization UB shenanagins), so I'm disabling this for
        // now until I get a second opinion.
        unsafe {
            let queue = &mut *UnsafeCell::get(&self.queue);
            return queue.pop_front();
        }
    }

    /*
    #[inline]
    pub fn is_empty(&self) -> bool {
        #[cfg(not(feature = "experimental_optimizations"))]
        return RefCell::borrow(&self.queue).is_empty();

        #[cfg(feature = "experimental_optimizations")]
        // SAFETY: This is single-threaded only, this Vec can only be borrowed in
        // the `send()` and `try_recv()` methods inside this module, and these methods
        // only borrow them for as long as the method lasts, so it isn't possible to
        // have a data race.
        //
        // That being said, I'm not 100% sure about the safety of this (there might
        // be some compiler optimization UB shenanagins), so I'm disabling this for
        // now until I get a second opinion.
        unsafe {
            let queue = &*UnsafeCell::get(&self.queue);
            return queue.is_empty();
        }
    }
    */
}

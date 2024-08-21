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

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// Construct a single-threaded only unbounded mpsc queue.
pub(crate) fn single_thread_mpsc_queue<T>(initial_capacity: usize) -> (Sender<T>, Receiver<T>) {
    let queue = Rc::new(RefCell::new(VecDeque::with_capacity(initial_capacity)));

    (
        Sender {
            queue: Rc::clone(&queue),
        },
        Receiver { queue },
    )
}

/// The sending end of a single-threaded only unbounded mpsc queue.
pub(crate) struct Sender<T> {
    queue: Rc<RefCell<VecDeque<T>>>,
}

impl<T> Sender<T> {
    #[inline]
    pub fn send(&mut self, msg: T) {
        RefCell::borrow_mut(&self.queue).push_back(msg);
    }

    #[inline]
    pub fn send_to_front(&mut self, msg: T) {
        RefCell::borrow_mut(&self.queue).push_front(msg);
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
    queue: Rc<RefCell<VecDeque<T>>>,
}

impl<T> Receiver<T> {
    #[inline]
    pub fn try_recv(&mut self) -> Option<T> {
        RefCell::borrow_mut(&self.queue).pop_front()
    }
}

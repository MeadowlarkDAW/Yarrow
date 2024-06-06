// ---------------------------------------------------------------------------------
//
//    '%%' '%% '%%'
//    %'%\% | %/%'%     Yarrow GUI Library
//        \ | /
//         \|/          https://codeberg.org/BillyDM/Yarrow
//          |
//
//
// MIT License Copyright (c) 2024 Billy Messenger
// https://github.com/MeadowlarkDAW/Yarrow/blob/main/LICENSE
//
// ---------------------------------------------------------------------------------

use std::sync::Arc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
};

pub fn action_channel<A: Clone + 'static>() -> (ActionSender<A>, ActionReceiver<A>) {
    let (sender, receiver) = mpsc::channel();
    (
        ActionSender {
            sender,
            action_sent: Arc::new(AtomicBool::new(false)),
        },
        ActionReceiver { receiver },
    )
}

#[derive(Clone)]
pub struct ActionSender<A: Clone + 'static> {
    pub sender: mpsc::Sender<A>,
    action_sent: Arc<AtomicBool>,
}

impl<A: Clone + 'static> ActionSender<A> {
    pub fn send(&mut self, action: impl Into<A>) -> Result<(), mpsc::SendError<A>> {
        self.action_sent.store(true, Ordering::Relaxed);
        self.sender.send(action.into())
    }

    pub(crate) fn any_action_sent(&mut self) -> bool {
        self.action_sent.swap(false, Ordering::Relaxed)
    }
}

pub struct ActionReceiver<A: Clone + 'static> {
    pub receiver: mpsc::Receiver<A>,
}

impl<A: Clone + 'static> ActionReceiver<A> {
    pub fn try_recv(&mut self) -> Result<A, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn try_iter(&mut self) -> mpsc::TryIter<A> {
        self.receiver.try_iter()
    }
}

pub trait Action: Clone + 'static {}

pub use self::inner::*;

#[cfg(not(feature = "crossbeam"))]
mod inner {
    use std::sync::mpsc;

    use super::Action;

    pub type SendError<A> = mpsc::SendError<A>;
    pub type TryRecvError = mpsc::TryRecvError;
    pub type TryIter<'a, A> = mpsc::TryIter<'a, A>;

    #[derive(Clone)]
    pub struct ActionSender<A: Action> {
        sender: mpsc::Sender<A>,
    }

    impl<A: Action> ActionSender<A> {
        pub fn send(&mut self, action: impl Into<A>) -> Result<(), SendError<A>> {
            self.sender.send(action.into())
        }
    }

    pub struct ActionReceiver<A: Action> {
        receiver: mpsc::Receiver<A>,
    }

    impl<A: Action> ActionReceiver<A> {
        pub fn try_recv(&mut self) -> Result<A, TryRecvError> {
            self.receiver.try_recv()
        }

        pub fn try_iter(&mut self) -> TryIter<A> {
            self.receiver.try_iter()
        }
    }

    pub fn action_channel<A: Action>() -> (ActionSender<A>, ActionReceiver<A>) {
        let (sender, receiver) = mpsc::channel();
        (ActionSender { sender }, ActionReceiver { receiver })
    }
}

#[cfg(feature = "crossbeam")]
mod inner {
    use super::Action;

    pub type SendError<A> = crossbeam_channel::SendError<A>;
    pub type TryRecvError = crossbeam_channel::TryRecvError;
    pub type TryIter<'a, A> = crossbeam_channel::TryIter<'a, A>;

    #[derive(Clone)]
    pub struct ActionSender<A: Action> {
        sender: crossbeam_channel::Sender<A>,
    }

    impl<A: Action> ActionSender<A> {
        pub fn send(&mut self, action: impl Into<A>) -> Result<(), SendError<A>> {
            self.sender.send(action.into())
        }
    }

    pub struct ActionReceiver<A: Action> {
        receiver: crossbeam_channel::Receiver<A>,
    }

    impl<A: Action> ActionReceiver<A> {
        pub fn try_recv(&mut self) -> Result<A, TryRecvError> {
            self.receiver.try_recv()
        }

        pub fn try_iter(&mut self) -> TryIter<A> {
            self.receiver.try_iter()
        }
    }

    pub fn action_channel<A: Action>() -> (ActionSender<A>, ActionReceiver<A>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        (ActionSender { sender }, ActionReceiver { receiver })
    }
}

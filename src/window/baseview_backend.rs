use std::error::Error;

use crate::action_queue::ActionSender;
use crate::application::{AppContext, Application};

struct AppHandler<A: Application> {
    user_app: A,
    context: AppContext<A::Action>,
    action_sender: ActionSender<A::Action>,
}

impl<A: Application> AppHandler<A> {
    pub fn new(
        mut user_app: A,
        action_sender: ActionSender<A::Action>,
    ) -> Result<Self, Box<dyn Error>> {
        let config = user_app.init()?;

        Ok(Self {
            user_app,
            context: AppContext::new(config),
            action_sender,
        })
    }
}

pub fn run_blocking<A: Application>(
    user_app: A,
    action_sender: ActionSender<A::Action>,
) -> Result<(), Box<dyn Error>> {
    let mut app_handler = AppHandler::new(user_app, action_sender)?;
    Ok(())
}

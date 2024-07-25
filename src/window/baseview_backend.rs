use crate::Application;

struct AppHandler<A: Application> {
    user_app: A,
}

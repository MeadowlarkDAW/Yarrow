pub fn main() {
    yarrow::run_blocking::<MyApp>(Default::default()).unwrap();
}

struct MyApp {}

impl yarrow::application::Application for MyApp {
    type Action = ();
}

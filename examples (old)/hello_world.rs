use yarrow::prelude::*;

pub fn main() {
    // Set up logging stuff.
    env_logger::init();

    yarrow::run_blocking::<MyApp>(AppConfig::default()).unwrap()
}

struct MyApp {
    hello_label: Label,
}

impl Application for MyApp {
    type Action = ();

    fn init(cx: &mut AppContext<Self::Action>) -> Result<Self, Box<dyn std::error::Error>> {
        // Each element has its own custom style struct.
        cx.res.style_system.add(
            ClassID::default(), // class ID
            true,               // is_dark_theme
            LabelStyle {
                back_quad: QuadStyle {
                    bg: background_rgb(100, 30, 80),
                    border: border(rgb(200, 60, 160), 2.0, radius(10.0)),
                    ..Default::default()
                },
                text_padding: padding_all_same(10.0),
                ..Default::default()
            },
        );

        // Elements are added to the view of a window context.
        let mut window_cx = cx.main_window();

        // The clear color of the window can be set at any time.
        window_cx.set_clear_color(rgb(20, 20, 20));

        let mut new_self = Self {
            // Most elements provide a builder style constructor.
            //
            // If no bounding rectangle is given, then by default the element has
            // a size of `0` meaning it is invisible. This allows us to layout out
            // the element later in a dedicated layout function.
            hello_label: Label::builder().text("Hello World!").build(&mut window_cx),
        };

        new_self.layout(&mut window_cx);

        Ok(new_self)
    }

    fn on_window_event(
        &mut self,
        event: AppWindowEvent,
        window_id: WindowID,
        cx: &mut AppContext<()>,
    ) {
        match event {
            AppWindowEvent::WindowResized => {
                if window_id == MAIN_WINDOW {
                    self.layout(&mut cx.main_window());
                }
            }
            _ => {}
        }
    }
}

impl MyApp {
    pub fn layout(&mut self, window_cx: &mut WindowContext<()>) {
        // You are in full control over how and when your elements are laid out,
        // styled, and mutated. You can be as fine-grained and optimized as you
        // like, however Yarrow is also designed to work in a sort-of
        // immediate-mode fasion for simplicity. Element handles send an update
        // to the update queue only when the data in the called methods differ
        // from its current state.

        // For layouts which depend on the size of some content, the calculated
        // size can be gotten from the handles.
        //
        // This calculated size is automatically cached, so don't worry about
        // it being too expensive to use in an immediate-mode fasion.
        let label_size = self.hello_label.desired_size(window_cx.res);

        // Center the label on the screen.
        let window_rect = Rect::from_size(window_cx.logical_size());
        let label_rect = centered_rect(window_rect.center(), label_size);

        // Element handles have a generic part with common methods.
        self.hello_label.set_rect(label_rect);
    }
}

use std::rc::Rc;
use yarrow::prelude::*;
// TODO: move to baseview backend

fn main() {
    // Set up logging stuff.
    env_logger::init();

    // Actions are sent via a regular Rust mpsc queue.
    let (action_sender, action_receiver) = yarrow::action_channel();

    yarrow::run_blocking(
        MyApp {
            _action_sender: action_sender.clone(),
            _action_receiver: action_receiver,
            main_window_elements: None,
        },
        action_sender,
    )
    .unwrap();
}

struct MyApp {
    _action_sender: ActionSender<()>,
    _action_receiver: ActionReceiver<()>,

    main_window_elements: Option<MainWindowElements>,
}

struct MainWindowElements {
    hello_label: Label,
}

impl Application for MyApp {
    type Action = ();

    fn on_window_event(
        &mut self,
        event: AppWindowEvent,
        window_id: WindowID,
        cx: &mut AppContext<()>,
    ) {
        match event {
            AppWindowEvent::WindowOpened => {
                // Yarrow has first-class mutli-window support.
                if window_id == MAIN_WINDOW {
                    self.build_main_window(cx);
                }
            }
            AppWindowEvent::WindowResized => {
                if window_id == MAIN_WINDOW {
                    let window_size = cx.window_context(MAIN_WINDOW).unwrap().logical_size();
                    self.layout_main_window(window_size);
                }
            }
            _ => {}
        }
    }
}

impl MyApp {
    fn build_main_window(&mut self, cx: &mut AppContext<()>) {
        // Each element has its own custom style struct.
        // Styles are wrapped in an `Rc` pointer for cheap cloning and diffing.
        let label_style = Rc::new(LabelStyle {
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(100, 30, 80, 255)),
                border: BorderStyle {
                    color: RGBA8::new(200, 60, 160, 255),
                    width: 2.0,
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Padding::new(10.0, 10.0, 10.0, 10.0),
            ..Default::default()
        });

        // Elements are added the the view of a window context.
        let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();

        // The clear color of the window can be set at any time.
        main_window_cx.view.clear_color = RGBA8::new(20, 20, 20, 255).into();

        // Most elements provide a builder style constructor.
        //
        // If no bounding rectangle is given, then by default the element has
        // a size of `0` meaning it is invisible. This allows us to layout out
        // the element later in a dedicated layout function.
        let hello_label = Label::builder(&label_style)
            .text("Hello World!")
            .build(&mut main_window_cx);

        self.main_window_elements = Some(MainWindowElements { hello_label });

        self.layout_main_window(main_window_cx.logical_size());
    }

    fn layout_main_window(&mut self, window_size: Size) {
        let Some(elements) = &mut self.main_window_elements else {
            return;
        };

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
        let label_size = elements.hello_label.desired_padded_size();

        // Center the label on the screen.
        let window_rect = Rect::from_size(window_size);
        let label_rect = centered_rect(window_rect.center(), label_size);

        // Element handles have a generic part with common methods.
        elements.hello_label.el.set_rect(label_rect);
    }
}

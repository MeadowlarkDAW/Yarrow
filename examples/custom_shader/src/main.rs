mod my_custom_element;

use my_custom_element::MyCustomElement;
use yarrow::prelude::*;

pub fn main() {
    let (action_sender, action_receiver) = yarrow::action_channel();

    yarrow::run_blocking(
        WindowConfig::default(),
        action_sender,
        action_receiver,
        || MyApp {
            main_window_elements: None,
        },
    )
    .unwrap();
}

struct MyApp {
    // Yarrow is designed to work even when a window is not currently
    // open (useful in an audio plugin context).
    main_window_elements: Option<MainWindowElements>,
}

// A struct to hold all of our elements that belong to the main window.
struct MainWindowElements {
    _my_custom_element: MyCustomElement,
}

impl MainWindowElements {
    pub fn build(cx: &mut WindowContext<'_, ()>) -> Self {
        Self {
            _my_custom_element: MyCustomElement::builder()
                .rect(rect(20.0, 20.0, 200.0, 200.0))
                .build(cx),
        }
    }
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
                if window_id == MAIN_WINDOW {
                    let mut cx = cx.window_context(MAIN_WINDOW).unwrap();
                    cx.view.clear_color = rgb(20, 20, 20).into();

                    self.main_window_elements = Some(MainWindowElements::build(&mut cx));
                }
            }
            _ => {}
        }
    }
}

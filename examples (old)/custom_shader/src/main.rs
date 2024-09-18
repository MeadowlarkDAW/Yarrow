mod my_custom_element;

use my_custom_element::MyCustomElement;
use yarrow::prelude::*;

pub fn main() {
    yarrow::run_blocking::<MyApp>(AppConfig::default()).unwrap();
}

struct MyApp {
    _my_custom_element: MyCustomElement,
}

impl Application for MyApp {
    type Action = ();

    fn init(cx: &mut AppContext<Self::Action>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut window_cx = cx.main_window();
        window_cx.set_clear_color(rgb(20, 20, 20));

        Ok(Self {
            _my_custom_element: MyCustomElement::builder()
                .rect(rect(20.0, 20.0, 200.0, 200.0))
                .build(&mut window_cx),
        })
    }
}

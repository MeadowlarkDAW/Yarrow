use yarrow::prelude::*;

/// Define a scissoring rectangle with a given ID.
///
/// This ID can be any `u32` value (expect for `u32::MAX` which is reserved
/// for the "default scissoring rectangle" that covers the whole window).
pub const CONTENT_SRECT: ScissorRectID = ScissorRectID(0);

struct MyApp {
    long_boi: TextInput,
    scroll_area: ScrollArea,
}

impl Application for MyApp {
    type Action = ();

    fn init(cx: &mut AppContext<Self::Action>) -> Result<Self, Box<dyn std::error::Error>> {
        yarrow::theme::yarrow_dark::load(Default::default(), &mut cx.res);

        let mut window_cx = cx.main_window();

        let mut new_self = Self {
            long_boi: TextInput::builder()
                .text("L0ng b0I")
                .scissor_rect(CONTENT_SRECT)
                .build(&mut window_cx),
            scroll_area: ScrollArea::builder()
                // Set the z index higher than the contents so that it has priority
                // on mouse events.
                .z_index(1)
                // Set the scissoring rectangle that this element should control.
                .control_scissor_rect(CONTENT_SRECT)
                // Note, do *NOT* assign the scroll area element itself to the scissoring
                // rectangle it controls, or it will not function properly.
                .scissor_rect(ScissorRectID::DEFAULT)
                .build(&mut window_cx),
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
    pub fn layout(&mut self, window_cx: &mut WindowContext<'_, ()>) {
        // Assign the scroll area element to fill the area we want (in this case the
        // entire window).
        self.scroll_area
            .set_rect(Rect::from_size(window_cx.logical_size()));

        // Layout the elements like normal.
        //
        // NOTE: The position of an element is relative to the origin of its assigned
        // scissoring rectangle. So if the position of the rectangle of `self.scroll_area`
        // was `(50.0, 70.0)`, then the position of this element will be offset by that
        // amount.
        self.long_boi.set_rect(rect(20.0, 20.0, 200.0, 1000.0));

        // Set the "content size" of the scroll area. In this case we want it to cover
        // the size of our elements with a bit of padding on the top and bottom.
        self.scroll_area.set_content_size(size(
            self.long_boi.max_x() + 20.0,
            self.long_boi.max_y() + 20.0,
        ));
    }
}

pub fn main() {
    yarrow::run_blocking::<MyApp>(AppConfig::default()).unwrap();
}

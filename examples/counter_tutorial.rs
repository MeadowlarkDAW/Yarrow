use yarrow::prelude::*;

#[derive(Clone)]
pub enum MyAction {
    OffsetCounterBy(i32),
    ResetCounter,
}

struct MyApp {
    hello_label: Label,
    increment_btn: Button,
    decrement_btn: Button,
    reset_btn: Button,

    style: MyStyle,

    count: i32,
}

impl MyApp {
    fn sync_state(&mut self, window_cx: &mut WindowContext<MyAction>) {
        let mut needs_layout = false;

        if self
            .hello_label
            .set_text(Some(&format!("{}", self.count)), window_cx.res)
        {
            // Changing the text may resize the label, so do a layout.
            needs_layout = true;
        }

        if needs_layout {
            self.layout(window_cx);
        }
    }

    fn layout(&mut self, window_cx: &mut WindowContext<MyAction>) {
        let label_size = self.hello_label.desired_size(window_cx.res);

        // Center the label inside the window
        let window_rect = Rect::from_size(window_cx.logical_size());
        let label_rect = centered_rect(window_rect.center(), label_size);

        self.hello_label.set_rect(label_rect);

        self.increment_btn.layout(
            point(
                self.style.window_padding.left,
                self.style.window_padding.top,
            ),
            window_cx.res,
        );
        self.decrement_btn.layout(
            point(
                self.increment_btn.max_x() + self.style.button_spacing,
                self.style.window_padding.top,
            ),
            window_cx.res,
        );
        self.reset_btn.layout(
            point(
                self.decrement_btn.max_x() + self.style.button_spacing,
                self.style.window_padding.top,
            ),
            window_cx.res,
        );
    }
}

impl Application for MyApp {
    type Action = MyAction;

    fn init(cx: &mut AppContext<Self::Action>) -> Result<Self, Box<dyn std::error::Error>> {
        let style = MyStyle::default();
        style.load(&mut cx.res);

        let mut window_cx = cx.main_window();

        let mut new_self = Self {
            hello_label: Label::builder()
                .class(MyStyle::CLASS_FANCY_LABEL)
                .text("Hello World!")
                .build(&mut window_cx),
            increment_btn: Button::builder()
                .text("+")
                .on_select(MyAction::OffsetCounterBy(1))
                .build(&mut window_cx),
            decrement_btn: Button::builder()
                .text("-")
                .on_select(MyAction::OffsetCounterBy(-1))
                .build(&mut window_cx),
            reset_btn: Button::builder()
                .text("reset")
                .on_select(MyAction::ResetCounter)
                .build(&mut window_cx),

            style,

            count: 0,
        };

        new_self.sync_state(&mut window_cx);

        Ok(new_self)
    }

    fn on_action_emitted(&mut self, cx: &mut AppContext<Self::Action>) {
        let mut state_changed = false;

        while let Ok(action) = cx.action_receiver.try_recv() {
            match action {
                MyAction::OffsetCounterBy(offset) => {
                    self.count += offset;
                    state_changed = true;
                }
                MyAction::ResetCounter => {
                    self.count = 0;
                    state_changed = true;
                }
            }
        }

        if state_changed {
            self.sync_state(&mut cx.main_window());
        }
    }

    fn on_window_event(
        &mut self,
        event: AppWindowEvent,
        window_id: WindowID,
        cx: &mut AppContext<MyAction>,
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

pub struct MyStyle {
    window_padding: Padding,
    button_spacing: f32,
}

impl Default for MyStyle {
    fn default() -> Self {
        Self {
            window_padding: padding_all_same(10.0),
            button_spacing: 8.0,
        }
    }
}

impl MyStyle {
    pub const CLASS_FANCY_LABEL: ClassID = 1;

    pub fn load(&self, res: &mut ResourceCtx) {
        yarrow::theme::yarrow_dark::load(Default::default(), res);

        res.style_system.add(
            Self::CLASS_FANCY_LABEL,
            true,
            LabelStyle {
                back_quad: QuadStyle {
                    bg: background_hex(0x641e50),
                    border: border(hex(0xc83ca0), 2.0, radius(10.0)),
                    ..Default::default()
                },
                text_padding: padding_all_same(10.0),
                ..Default::default()
            },
        );
    }
}

pub fn main() {
    yarrow::run_blocking::<MyApp>(AppConfig::default()).unwrap();
}

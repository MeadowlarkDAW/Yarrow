use yarrow::prelude::*;

#[derive(Clone)]
pub enum MyAction {
    OffsetCounterBy(i32),
    ResetCounter,
}

#[derive(Default)]
struct MyApp {
    main_window_elements: Option<MainWindowElements>,
    style: MyStyle,

    count: i32,
}

impl MyApp {
    pub fn sync_state(&mut self, cx: &mut WindowContext<'_, MyAction>) {
        let Some(elements) = &mut self.main_window_elements else {
            return;
        };

        let mut needs_layout = false;

        if elements
            .hello_label
            .set_text(Some(&format!("{}", self.count)), cx.res)
        {
            // Changing the text may resize the label, so do a layout.
            needs_layout = true;
        }

        if needs_layout {
            elements.layout(&self.style, cx);
        }
    }
}

impl Application for MyApp {
    type Action = MyAction;

    fn on_action_emitted(&mut self, cx: &mut AppContext<Self::Action>) {
        let Some(mut cx) = cx.window_context(MAIN_WINDOW) else {
            return;
        };

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
            self.sync_state(&mut cx);
        }
    }

    fn on_window_event(
        &mut self,
        event: AppWindowEvent,
        window_id: WindowID,
        cx: &mut AppContext<MyAction>,
    ) {
        match event {
            AppWindowEvent::WindowOpened => {
                if window_id == MAIN_WINDOW {
                    self.style.load(&mut cx.res);

                    let mut cx = cx.window_context(MAIN_WINDOW).unwrap();

                    self.main_window_elements = Some(MainWindowElements::build(&mut cx));

                    self.sync_state(&mut cx);
                }
            }
            AppWindowEvent::WindowResized => {
                if window_id == MAIN_WINDOW {
                    let mut cx = cx.window_context(MAIN_WINDOW).unwrap();

                    self.main_window_elements
                        .as_mut()
                        .unwrap()
                        .layout(&self.style, &mut cx);
                }
            }
            _ => {}
        }
    }
}

pub struct MainWindowElements {
    hello_label: Label,
    increment_btn: Button,
    decrement_btn: Button,
    reset_btn: Button,
}

impl MainWindowElements {
    pub fn build(cx: &mut WindowContext<'_, MyAction>) -> Self {
        Self {
            hello_label: Label::builder()
                .class(MyStyle::CLASS_FANCY_LABEL)
                .text("Hello World!")
                .build(cx),
            increment_btn: Button::builder()
                .text("+")
                .on_select(MyAction::OffsetCounterBy(1))
                .build(cx),
            decrement_btn: Button::builder()
                .text("-")
                .on_select(MyAction::OffsetCounterBy(-1))
                .build(cx),
            reset_btn: Button::builder()
                .text("reset")
                .on_select(MyAction::ResetCounter)
                .build(cx),
        }
    }

    pub fn layout(&mut self, style: &MyStyle, cx: &mut WindowContext<'_, MyAction>) {
        let label_size = self.hello_label.desired_size(cx.res);

        // Center the label inside the window
        let window_rect = Rect::from_size(cx.logical_size());
        let label_rect = centered_rect(window_rect.center(), label_size);

        self.hello_label.el.set_rect(label_rect);

        self.increment_btn.layout(
            point(style.window_padding.left, style.window_padding.top),
            cx.res,
        );
        self.decrement_btn.layout(
            point(
                self.increment_btn.el.rect().max_x() + style.button_spacing,
                style.window_padding.top,
            ),
            cx.res,
        );
        self.reset_btn.layout(
            point(
                self.decrement_btn.el.rect().max_x() + style.button_spacing,
                style.window_padding.top,
            ),
            cx.res,
        );
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
                },
                text_padding: padding_all_same(10.0),
                ..Default::default()
            },
        );
    }
}

pub fn main() {
    let (action_sender, action_receiver) = yarrow::action_channel();

    yarrow::run_blocking(MyApp::default(), action_sender, action_receiver).unwrap();
}

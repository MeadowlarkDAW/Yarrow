#[path = "gallery/basic_elements.rs"]
mod basic_elements;
#[path = "gallery/style.rs"]
mod style;

use self::style::MyStyle;
use yarrow::prelude::*;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, parse_display::Display)]
#[display(style = "Title Case")]
enum MyTab {
    BasicElements,
    #[display("Knobs & Sliders")]
    KnobsAndSliders,
    More,
}
impl MyTab {
    pub const ALL: [Self; 3] = [Self::BasicElements, Self::KnobsAndSliders, Self::More];
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, parse_display::Display)]
#[display(style = "Title Case")]
enum MenuOption {
    Hello,
    World,
    About,
}
impl MenuOption {
    pub const ALL: [Self; 3] = [Self::Hello, Self::World, Self::About];

    pub fn right_text(&self) -> &'static str {
        match self {
            Self::Hello => "Ctrl+H",
            Self::World => "Ctrl+W",
            Self::About => "",
        }
    }
}

const MAIN_Z_INDEX: ZIndex = 10;
const SCROLL_AREA_Z_INDEX: ZIndex = 20;
const OVERLAY_Z_INDEX: ZIndex = 30;

pub fn main() {
    // Set up logging stuff.
    env_logger::init();

    // Actions are sent via a regular Rust mpsc queue.
    let (action_sender, action_receiver) = yarrow::action_channel();

    yarrow::run_blocking(
        MyApp::new(action_sender.clone(), action_receiver),
        action_sender,
    )
    .unwrap();
}

#[derive(Debug, Clone, PartialEq)]
enum MyAction {
    BasicElements(basic_elements::Action),
    LeftPanelResized(f32),
    LeftPanelResizeFinished(f32),
    MenuItemSelected(MenuOption),
    OpenMenu,
    ShowTooltip((TooltipInfo, WindowID)),
    HideTooltip(WindowID),
    TabSelected(MyTab),
}

impl From<basic_elements::Action> for MyAction {
    fn from(a: basic_elements::Action) -> Self {
        MyAction::BasicElements(a)
    }
}

// A struct to hold all of our elements that belong to the main window.
struct MainWindowElements {
    basic_elements: basic_elements::Elements,
    menu: DropDownMenu,
    menu_btn: Button,
    top_panel_bg: QuadElement,
    top_panel_border: QuadElement,
    left_panel_bg: QuadElement,
    left_panel_border: QuadElement,
    left_panel_resize_handle: ResizeHandle,
    tab_group: TabGroup,
    tooltip: Tooltip,
}

struct MyApp {
    _action_sender: ActionSender<MyAction>,
    action_receiver: ActionReceiver<MyAction>,

    // Yarrow is designed to work even when a window is not currently
    // open (useful in an audio plugin context).
    main_window_elements: Option<MainWindowElements>,

    style: MyStyle,
    did_load_fonts: bool,
}

impl MyApp {
    fn new(
        action_sender: ActionSender<MyAction>,
        action_receiver: ActionReceiver<MyAction>,
    ) -> Self {
        Self {
            _action_sender: action_sender,
            action_receiver,
            main_window_elements: None,
            style: MyStyle::new(),
            did_load_fonts: false,
        }
    }

    fn build_main_window(&mut self, cx: &mut WindowContext<'_, MyAction>) {
        cx.view.clear_color = self.style.clear_color.into();
        cx.view.set_num_additional_scissor_rects(1);

        let top_panel_bg = QuadElement::builder(&self.style.panel_bg_style).build(cx);
        let top_panel_border = QuadElement::builder(&self.style.panel_border_style).build(cx);

        let left_panel_bg = QuadElement::builder(&self.style.panel_bg_style).build(cx);
        let left_panel_border = QuadElement::builder(&self.style.panel_border_style).build(cx);

        let left_panel_resize_handle = ResizeHandle::builder(&self.style.resize_handle_style)
            .on_resized(|new_span| MyAction::LeftPanelResized(new_span))
            .on_resize_finished(|new_span| MyAction::LeftPanelResizeFinished(new_span))
            .min_span(100.0)
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let menu_btn = Button::builder(&self.style.menu_btn_style)
            .text('\u{f0c9}')
            .on_select(MyAction::OpenMenu)
            .z_index(MAIN_Z_INDEX)
            .build(cx);
        let menu = DropDownMenu::builder(&self.style.menu_style)
            .entries(vec![
                MenuEntry::Option {
                    left_text: format!("{}", MenuOption::Hello),
                    right_text: MenuOption::Hello.right_text().into(),
                    unique_id: 0,
                },
                MenuEntry::Option {
                    left_text: format!("{}", MenuOption::World),
                    right_text: MenuOption::World.right_text().into(),
                    unique_id: 1,
                },
                MenuEntry::Divider,
                MenuEntry::Option {
                    left_text: format!("{}", MenuOption::About),
                    right_text: MenuOption::About.right_text().into(),
                    unique_id: 2,
                },
            ])
            .on_entry_selected(|id| MyAction::MenuItemSelected(MenuOption::ALL[id]))
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let tooltip = Tooltip::builder(&self.style.tooltip_style)
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let tab_group = TabGroup::new(
            MyTab::ALL
                .map(|t| TabGroupOption::new(format!("{t}"), format!("{t}"), Point::default())),
            0,
            |i| MyAction::TabSelected(MyTab::ALL[i]),
            &self.style.tab_style,
            MAIN_Z_INDEX,
            IndicatorLinePlacement::Left,
            Align2::CENTER_RIGHT,
            MAIN_SCISSOR_RECT,
            cx,
        );

        self.main_window_elements = Some(MainWindowElements {
            basic_elements: basic_elements::Elements::new(&self.style, cx),
            menu,
            menu_btn,
            top_panel_bg,
            top_panel_border,
            left_panel_bg,
            left_panel_border,
            left_panel_resize_handle,
            tab_group,
            tooltip,
        });
    }

    fn layout_main_window(&mut self, cx: &mut WindowContext<'_, MyAction>) {
        let Some(elements) = &mut self.main_window_elements else {
            return;
        };

        let window_size = cx.logical_size();

        elements.top_panel_bg.el.set_rect(Rect::new(
            Point::zero(),
            Size::new(window_size.width, self.style.top_panel_height),
        ));
        elements.top_panel_border.el.set_rect(Rect::new(
            Point::new(
                0.0,
                self.style.top_panel_height - self.style.panel_border_width,
            ),
            Size::new(window_size.width, self.style.panel_border_width),
        ));

        let left_panel_width = elements.left_panel_resize_handle.current_span();
        elements.left_panel_bg.el.set_rect(Rect::new(
            Point::new(0.0, self.style.top_panel_height),
            Size::new(
                left_panel_width,
                window_size.height - self.style.top_panel_height,
            ),
        ));
        elements.left_panel_border.el.set_rect(Rect::new(
            Point::new(
                left_panel_width - self.style.panel_border_width,
                self.style.top_panel_height,
            ),
            Size::new(
                self.style.panel_border_width,
                window_size.height - self.style.top_panel_height,
            ),
        ));
        elements
            .left_panel_resize_handle
            .set_layout(ResizeHandleLayout {
                anchor: Point::new(0.0, self.style.top_panel_height),
                length: window_size.height,
            });

        let menu_btn_size = elements.menu_btn.desired_padded_size();
        let menu_btn_rect = Rect::new(
            Point::new(
                self.style.menu_btn_padding,
                (self.style.top_panel_height - menu_btn_size.height) * 0.5,
            ),
            menu_btn_size,
        );
        elements.menu_btn.el.set_rect(menu_btn_rect);

        elements
            .menu
            .set_position(Point::new(menu_btn_rect.min_x(), menu_btn_rect.max_y()));

        elements.tab_group.layout(
            Point::new(
                0.0,
                self.style.top_panel_height + self.style.tab_group_padding,
            ),
            self.style.tag_group_spacing,
            LayoutDirection::Vertical,
            Some(left_panel_width - self.style.panel_border_width),
        );

        elements.basic_elements.layout(
            Rect::new(
                Point::new(left_panel_width, self.style.top_panel_height),
                Size::new(
                    window_size.width - left_panel_width,
                    window_size.height - self.style.top_panel_height,
                ),
            ),
            &self.style,
            cx,
        )
    }

    fn handle_action(&mut self, action: MyAction, cx: &mut AppContext<MyAction>) {
        dbg!(&action);

        let Some(elements) = self.main_window_elements.as_mut() else {
            return;
        };

        let mut needs_layout = false;

        match action {
            MyAction::BasicElements(action) => {
                let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();
                needs_layout = elements
                    .basic_elements
                    .handle_action(action, &mut main_window_cx);
            }
            MyAction::LeftPanelResized(_new_span) => {
                needs_layout = true;
            }
            MyAction::LeftPanelResizeFinished(_new_span) => {}
            MyAction::MenuItemSelected(_option) => {}
            MyAction::OpenMenu => {
                elements.menu.open(None);
            }
            MyAction::ShowTooltip((info, _window_id)) => {
                elements.tooltip.show(
                    &info.message,
                    info.element_bounds,
                    info.align,
                    &mut cx.font_system,
                );
            }
            MyAction::HideTooltip(_window_id) => {
                elements.tooltip.hide();
            }
            MyAction::TabSelected(tab) => {
                elements.tab_group.updated_selected(tab as usize);
                elements
                    .basic_elements
                    .set_hidden(tab != MyTab::BasicElements);
            }
        }

        if needs_layout {
            let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();
            self.layout_main_window(&mut main_window_cx);
        }
    }
}

impl Application for MyApp {
    type Action = MyAction;

    fn main_window_config(&self) -> WindowConfig {
        WindowConfig {
            title: String::from("Yarrow Gallery Demo"),
            size: Size::new(700.0, 400.0),
            ..Default::default()
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
                // Yarrow has first-class mutli-window support.
                if window_id == MAIN_WINDOW {
                    if !self.did_load_fonts {
                        self.did_load_fonts = true;
                        self.style.load_fonts(&mut cx.font_system);
                    }

                    let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();

                    self.build_main_window(&mut main_window_cx);
                    self.layout_main_window(&mut main_window_cx);

                    main_window_cx.view.set_tooltip_actions(
                        |info| MyAction::ShowTooltip((info, MAIN_WINDOW)),
                        || MyAction::HideTooltip(MAIN_WINDOW),
                    );
                }
            }
            AppWindowEvent::WindowResized => {
                if window_id == MAIN_WINDOW {
                    let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();
                    self.layout_main_window(&mut main_window_cx);
                }
            }
            _ => {}
        }
    }

    fn on_action_emitted(&mut self, cx: &mut AppContext<Self::Action>) {
        while let Ok(action) = self.action_receiver.try_recv() {
            self.handle_action(action, cx);
        }
    }

    fn on_keyboard_event(
        &mut self,
        event: KeyboardEvent,
        window_id: WindowID,
        _cx: &mut AppContext<Self::Action>,
    ) {
        if window_id == MAIN_WINDOW
            && event.state == KeyState::Down
            && event.code == Code::KeyA
            && event.modifiers.ctrl()
            && !event.repeat
        {
            println!("program-wide keyboard shortcut activated: Ctrl+A")
        }
    }
}

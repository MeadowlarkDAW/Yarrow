#[path = "gallery/about_window.rs"]
mod about_window;
#[path = "gallery/basic_elements.rs"]
mod basic_elements;
#[path = "gallery/knobs_and_sliders.rs"]
mod knobs_and_sliders;
#[path = "gallery/style.rs"]
mod style;

use self::style::MyStyle;
use style::MyIcon;
use yarrow::prelude::*;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
enum MyTab {
    #[display(fmt = "Basic Elements")]
    BasicElements,
    #[display(fmt = "Knobs & Sliders")]
    KnobsAndSliders,
    More,
}
impl MyTab {
    pub const ALL: [Self; 3] = [Self::BasicElements, Self::KnobsAndSliders, Self::More];
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
enum MenuOption {
    Hello,
    World,
    About,
}
impl MenuOption {
    pub const ALL: [Self; 3] = [Self::Hello, Self::World, Self::About];

    pub fn right_text(&self) -> Option<&'static str> {
        match self {
            Self::Hello => None,
            Self::World => None,
            Self::About => Some("Ctrl+A"),
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
    KnobsAndSliders(knobs_and_sliders::Action),
    AboutWindow(about_window::Action),
    LeftPanelResized(f32),
    LeftPanelResizeFinished(f32),
    MenuItemSelected(MenuOption),
    OpenMenu,
    ShowTooltip((TooltipInfo, WindowID)),
    HideTooltip(WindowID),
    TabSelected(MyTab),
    OpenAboutWindow,
}

impl From<basic_elements::Action> for MyAction {
    fn from(a: basic_elements::Action) -> Self {
        MyAction::BasicElements(a)
    }
}

impl From<knobs_and_sliders::Action> for MyAction {
    fn from(a: knobs_and_sliders::Action) -> Self {
        MyAction::KnobsAndSliders(a)
    }
}

impl From<about_window::Action> for MyAction {
    fn from(a: about_window::Action) -> Self {
        MyAction::AboutWindow(a)
    }
}

// A struct to hold all of our elements that belong to the main window.
struct MainWindowElements {
    basic_elements: basic_elements::Elements,
    knobs_and_sliders: knobs_and_sliders::Elements,
    menu: DropDownMenu,
    menu_btn: IconButton,
    top_panel_bg: QuadElement,
    top_panel_border: QuadElement,
    left_panel_bg: QuadElement,
    left_panel_border: QuadElement,
    left_panel_resize_handle: ResizeHandle,
    tab_group: TabGroup,
    tooltip: Tooltip,
}

struct MyApp {
    action_sender: ActionSender<MyAction>,
    action_receiver: ActionReceiver<MyAction>,

    // Yarrow is designed to work even when a window is not currently
    // open (useful in an audio plugin context).
    main_window_elements: Option<MainWindowElements>,
    about_window_elements: Option<about_window::Elements>,

    style: MyStyle,
    did_load_resources: bool,
    current_tab: MyTab,
}

impl MyApp {
    fn new(
        action_sender: ActionSender<MyAction>,
        action_receiver: ActionReceiver<MyAction>,
    ) -> Self {
        Self {
            action_sender: action_sender,
            action_receiver,
            main_window_elements: None,
            about_window_elements: None,
            style: MyStyle::new(),
            did_load_resources: false,
            current_tab: MyTab::BasicElements,
        }
    }

    fn build_main_window(&mut self, cx: &mut WindowContext<'_, MyAction>) {
        cx.view.clear_color = self.style.clear_color.into();
        cx.view.set_num_additional_scissor_rects(2);

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

        let menu_btn = IconButton::builder(&self.style.menu_btn_style)
            .icon(MyIcon::Menu)
            .on_select(MyAction::OpenMenu)
            .z_index(MAIN_Z_INDEX)
            .build(cx);
        let menu = DropDownMenu::builder(&self.style.menu_style)
            .entries(vec![
                MenuEntry::option_with_right_text("Hello", MenuOption::Hello.right_text(), 0),
                MenuEntry::option_with_right_text("World", MenuOption::World.right_text(), 1),
                MenuEntry::option_with_right_text("About", MenuOption::About.right_text(), 2),
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
            self.current_tab as usize,
            |i| MyAction::TabSelected(MyTab::ALL[i]),
            &self.style.tab_style,
            MAIN_Z_INDEX,
            IndicatorLinePlacement::Left,
            Align2::CENTER_RIGHT,
            MAIN_SCISSOR_RECT,
            cx,
        );

        let mut basic_elements = basic_elements::Elements::new(&self.style, cx);
        let mut knobs_and_sliders = knobs_and_sliders::Elements::new(&self.style, cx);

        basic_elements.set_hidden(self.current_tab != MyTab::BasicElements);
        knobs_and_sliders.set_hidden(self.current_tab != MyTab::KnobsAndSliders);

        self.main_window_elements = Some(MainWindowElements {
            basic_elements,
            knobs_and_sliders,
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

    fn handle_action(&mut self, action: MyAction, cx: &mut AppContext<MyAction>) {
        #[cfg(debug_assertions)]
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
            MyAction::KnobsAndSliders(action) => {
                let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();
                needs_layout = elements.knobs_and_sliders.handle_action(
                    action,
                    &self.style,
                    &mut main_window_cx,
                );
            }
            MyAction::AboutWindow(action) => {
                if let Some(mut about_window_cx) = cx.window_context(about_window::ABOUT_WINDOW_ID)
                {
                    let close_about_window = self
                        .about_window_elements
                        .as_mut()
                        .unwrap()
                        .handle_action(action, &mut about_window_cx);

                    if close_about_window {
                        cx.close_window(about_window::ABOUT_WINDOW_ID);
                    }
                }
            }
            MyAction::LeftPanelResized(_new_span) => {
                needs_layout = true;
            }
            MyAction::LeftPanelResizeFinished(_new_span) => {}
            MyAction::MenuItemSelected(option) => {
                if option == MenuOption::About {
                    self.action_sender.send(MyAction::OpenAboutWindow).unwrap();
                }
            }
            MyAction::OpenMenu => {
                elements.menu.open(None);
            }
            MyAction::ShowTooltip((info, _window_id)) => {
                elements
                    .tooltip
                    .show(&info.message, info.element_bounds, info.align, &mut cx.res);
            }
            MyAction::HideTooltip(_window_id) => {
                elements.tooltip.hide();
            }
            MyAction::TabSelected(tab) => {
                self.current_tab = tab;
                elements
                    .tab_group
                    .updated_selected(tab as usize, &mut cx.res);
                elements
                    .basic_elements
                    .set_hidden(tab != MyTab::BasicElements);
                elements
                    .knobs_and_sliders
                    .set_hidden(tab != MyTab::KnobsAndSliders);
                needs_layout = true;
            }
            MyAction::OpenAboutWindow => {
                cx.open_window(about_window::ABOUT_WINDOW_ID, about_window::window_config());
            }
        }

        if needs_layout {
            let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();
            self.layout_main_window(&mut main_window_cx);
        }
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

        elements.menu_btn.layout_aligned(
            Point::new(
                self.style.menu_btn_padding,
                self.style.top_panel_height * 0.5,
            ),
            Align2::CENTER_LEFT,
        );

        elements.menu.set_position(Point::new(
            elements.menu_btn.el.rect().min_x(),
            elements.menu_btn.el.rect().max_y(),
        ));

        elements.tab_group.layout(
            Point::new(
                0.0,
                self.style.top_panel_height + self.style.tab_group_padding,
            ),
            self.style.tag_group_spacing,
            LayoutDirection::Vertical,
            Some(left_panel_width - self.style.panel_border_width),
        );

        let content_rect = Rect::new(
            Point::new(left_panel_width, self.style.top_panel_height),
            Size::new(
                window_size.width - left_panel_width,
                window_size.height - self.style.top_panel_height,
            ),
        );

        match self.current_tab {
            MyTab::BasicElements => {
                elements
                    .basic_elements
                    .layout(content_rect, &self.style, cx);
            }
            MyTab::KnobsAndSliders => {
                elements
                    .knobs_and_sliders
                    .layout(content_rect, &self.style, cx);
            }
            MyTab::More => {}
        }
    }
}

impl Application for MyApp {
    type Action = MyAction;

    fn main_window_config(&self) -> WindowConfig {
        WindowConfig {
            title: String::from("Yarrow Gallery Demo"),
            size: Size::new(700.0, 425.0),
            //scale_factor: ScaleFactorConfig::Custom(1.0.into()),
            /*
            surface_config: rootvg::surface::DefaultSurfaceConfig {
                instance_descriptor: wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::GL,
                    ..Default::default()
                },
                ..Default::default()
            },
            */
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
                    if !self.did_load_resources {
                        self.did_load_resources = true;
                        self.style.load_resources(&mut cx.res);
                    }

                    let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();

                    self.build_main_window(&mut main_window_cx);
                    self.layout_main_window(&mut main_window_cx);

                    main_window_cx.view.set_tooltip_actions(
                        |info| MyAction::ShowTooltip((info, MAIN_WINDOW)),
                        || MyAction::HideTooltip(MAIN_WINDOW),
                    );
                } else if window_id == about_window::ABOUT_WINDOW_ID {
                    let mut about_window_cx =
                        cx.window_context(about_window::ABOUT_WINDOW_ID).unwrap();

                    self.about_window_elements = Some(about_window::Elements::new(
                        &self.style,
                        &mut about_window_cx,
                    ));
                }
            }
            AppWindowEvent::WindowResized => {
                if window_id == MAIN_WINDOW {
                    let mut main_window_cx = cx.window_context(MAIN_WINDOW).unwrap();
                    self.layout_main_window(&mut main_window_cx);
                }
            }
            AppWindowEvent::WindowClosed => {
                if window_id == about_window::ABOUT_WINDOW_ID {
                    // When a window is closed, all handles to elements belonging to the
                    // window are invalidated.
                    self.about_window_elements = None;
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
            println!("program-wide keyboard shortcut activated: Ctrl+A");
            self.action_sender.send(MyAction::OpenAboutWindow).unwrap();
        }
    }
}

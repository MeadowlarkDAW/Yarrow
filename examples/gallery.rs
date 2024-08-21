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
    #[display("Basic Elements")]
    BasicElements,
    #[display("Knobs & Sliders")]
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

const RIGHT_CLICK_AREA_Z_INDEX: ZIndex = 0;
const MAIN_Z_INDEX: ZIndex = 10;
const SCROLL_AREA_Z_INDEX: ZIndex = 20;
const OVERLAY_Z_INDEX: ZIndex = 30;

pub fn main() {
    // Set up logging stuff.
    env_logger::init();

    // Actions are sent via a regular Rust mpsc queue.
    let (action_sender, action_receiver) = yarrow::action_channel();

    yarrow::run_blocking(MyApp::new(), action_sender, action_receiver).unwrap();
}

#[derive(Default, Debug, Clone, PartialEq)]
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
    #[default]
    None, // A quirk needed to get Yarrow's macros to work
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
    // Yarrow is designed to work even when a window is not currently
    // open (useful in an audio plugin context).
    main_window_elements: Option<MainWindowElements>,
    about_window_elements: Option<about_window::Elements>,

    style: MyStyle,
    did_load_resources: bool,
    current_tab: MyTab,
}

impl MyApp {
    fn new() -> Self {
        Self {
            main_window_elements: None,
            about_window_elements: None,
            style: MyStyle::new(),
            did_load_resources: false,
            current_tab: MyTab::BasicElements,
        }
    }

    fn build_main_window(&mut self, cx: &mut WindowContext<'_, MyAction>) {
        cx.view.clear_color = self.style.clear_color.into();

        // Push the main Z index onto the stack to make it the default.
        cx.push_z_index(MAIN_Z_INDEX);

        let top_panel_bg = QuadElement::builder().class(CLASS_PANEL).build(cx);
        let top_panel_border = QuadElement::builder()
            .class(MyStyle::CLASS_PANEL_BORDER)
            .build(cx);

        let left_panel_bg = QuadElement::builder().class(CLASS_PANEL).build(cx);
        let left_panel_border = QuadElement::builder()
            .class(MyStyle::CLASS_PANEL_BORDER)
            .build(cx);

        let left_panel_resize_handle = ResizeHandle::builder()
            .on_resized(|new_span| MyAction::LeftPanelResized(new_span))
            .on_resize_finished(|new_span| MyAction::LeftPanelResizeFinished(new_span))
            .min_span(100.0)
            // If a z index or scissoring rect ID is set in an element builder, it will
            // override the default one in `cx`.
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let menu_btn = Button::builder()
            .class(CLASS_MENU)
            .icon(MyIcon::Menu)
            .on_select(MyAction::OpenMenu)
            .build(cx);
        let menu = DropDownMenu::builder()
            .entries(vec![
                MenuEntry::option_with_right_text("Hello", MenuOption::Hello.right_text(), 0),
                MenuEntry::option_with_right_text("World", MenuOption::World.right_text(), 1),
                MenuEntry::Divider,
                MenuEntry::option_with_right_text("About", MenuOption::About.right_text(), 2),
            ])
            .on_entry_selected(|id| MyAction::MenuItemSelected(MenuOption::ALL[id]))
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let tooltip = Tooltip::builder().z_index(OVERLAY_Z_INDEX).build(cx);

        let tab_group = TabGroup::new(
            MyTab::ALL
                .map(|t| TabGroupOption::new(Some(format!("{t}")), None, Some(format!("{t}")))),
            self.current_tab as usize,
            |i| MyAction::TabSelected(MyTab::ALL[i]),
            None,
            IndicatorLinePlacement::Left,
            Align2::CENTER_RIGHT,
            None,
            None,
            cx,
        );

        let mut basic_elements = basic_elements::Elements::new(cx);
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
            MyAction::None => {}
            MyAction::BasicElements(action) => {
                let mut cx = cx.window_context(MAIN_WINDOW).unwrap();
                needs_layout = elements.basic_elements.handle_action(action, &mut cx);
            }
            MyAction::KnobsAndSliders(action) => {
                let mut cx = cx.window_context(MAIN_WINDOW).unwrap();
                needs_layout =
                    elements
                        .knobs_and_sliders
                        .handle_action(action, &self.style, &mut cx);
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
                    cx.action_sender.send(MyAction::OpenAboutWindow).unwrap();
                }
            }
            MyAction::OpenMenu => {
                elements.menu.open(None);
            }
            MyAction::ShowTooltip((info, _window_id)) => {
                elements
                    .tooltip
                    .show(&info.text, info.align, info.element_bounds, &mut cx.res);
            }
            MyAction::HideTooltip(_window_id) => {
                elements.tooltip.hide();
            }
            MyAction::TabSelected(tab) => {
                self.current_tab = tab;
                elements.tab_group.updated_selected(tab as usize);
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
            let mut cx = cx.window_context(MAIN_WINDOW).unwrap();
            self.layout_main_window(&mut cx);
        }
    }

    fn layout_main_window(&mut self, cx: &mut WindowContext<'_, MyAction>) {
        let Some(elements) = &mut self.main_window_elements else {
            return;
        };

        let window_size = cx.logical_size();

        elements.top_panel_bg.set_rect(rect(
            0.0,
            0.0,
            window_size.width,
            self.style.top_panel_height,
        ));
        elements.top_panel_border.set_rect(rect(
            0.0,
            self.style.top_panel_height - self.style.panel_border_width,
            window_size.width,
            self.style.panel_border_width,
        ));

        let left_panel_width = elements.left_panel_resize_handle.current_span();
        elements.left_panel_bg.set_rect(rect(
            0.0,
            self.style.top_panel_height,
            left_panel_width,
            window_size.height - self.style.top_panel_height,
        ));
        elements.left_panel_border.set_rect(rect(
            left_panel_width - self.style.panel_border_width,
            self.style.top_panel_height,
            self.style.panel_border_width,
            window_size.height - self.style.top_panel_height,
        ));
        elements
            .left_panel_resize_handle
            .set_layout(ResizeHandleLayout {
                anchor: point(0.0, self.style.top_panel_height),
                length: window_size.height,
            });

        elements.menu_btn.layout_aligned(
            point(
                self.style.menu_btn_padding,
                self.style.top_panel_height * 0.5,
            ),
            Align2::CENTER_LEFT,
            cx.res,
        );

        elements
            .menu
            .set_position(point(elements.menu_btn.min_x(), elements.menu_btn.max_y()));

        elements.tab_group.layout(
            point(
                0.0,
                self.style.top_panel_height + self.style.tab_group_padding,
            ),
            self.style.tag_group_spacing,
            LayoutDirection::Vertical,
            Some(left_panel_width - self.style.panel_border_width),
            cx.res,
        );

        let content_rect = rect(
            left_panel_width,
            self.style.top_panel_height,
            window_size.width - left_panel_width,
            window_size.height - self.style.top_panel_height,
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
                        self.style.load(&mut cx.res);
                    }

                    let mut cx = cx.window_context(MAIN_WINDOW).unwrap();

                    self.build_main_window(&mut cx);
                    self.layout_main_window(&mut cx);

                    cx.view.set_tooltip_actions(
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
                    let mut cx = cx.window_context(MAIN_WINDOW).unwrap();
                    self.layout_main_window(&mut cx);
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
        while let Ok(action) = cx.action_receiver.try_recv() {
            self.handle_action(action, cx);
        }
    }

    fn on_keyboard_event(
        &mut self,
        event: KeyboardEvent,
        window_id: WindowID,
        cx: &mut AppContext<Self::Action>,
    ) {
        if window_id == MAIN_WINDOW
            && event.state == KeyState::Down
            && event.code == Code::KeyA
            && event.modifiers.ctrl()
            && !event.repeat
        {
            println!("program-wide keyboard shortcut activated: Ctrl+A");
            cx.action_sender.send(MyAction::OpenAboutWindow).unwrap();
        }
    }
}

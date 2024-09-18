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

    yarrow::run_blocking::<MyApp>(AppConfig {
        main_window_config: WindowConfig {
            title: String::from("Yarrow Gallery Demo"),
            size: Size::new(700.0, 425.0),
            ..Default::default()
        },
        ..Default::default()
    })
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
    main_window_elements: MainWindowElements,
    about_window_elements: Option<about_window::Elements>,

    style: MyStyle,
    current_tab: MyTab,
}

impl Application for MyApp {
    type Action = MyAction;

    fn init(cx: &mut AppContext<Self::Action>) -> Result<Self, Box<dyn std::error::Error>> {
        let style = MyStyle::new();
        style.load(&mut cx.res);

        let mut window_cx = cx.main_window();

        window_cx.set_clear_color(style.clear_color);

        // Push the main Z index onto the stack to make it the default.
        window_cx.push_z_index(MAIN_Z_INDEX);

        let current_tab = MyTab::BasicElements;

        let top_panel_bg = QuadElement::builder()
            .class(CLASS_PANEL)
            .build(&mut window_cx);
        let top_panel_border = QuadElement::builder()
            .class(MyStyle::CLASS_PANEL_BORDER)
            .build(&mut window_cx);

        let left_panel_bg = QuadElement::builder()
            .class(CLASS_PANEL)
            .build(&mut window_cx);
        let left_panel_border = QuadElement::builder()
            .class(MyStyle::CLASS_PANEL_BORDER)
            .build(&mut window_cx);

        let left_panel_resize_handle = ResizeHandle::builder()
            .on_resized(|new_span| MyAction::LeftPanelResized(new_span))
            .on_resize_finished(|new_span| MyAction::LeftPanelResizeFinished(new_span))
            .min_span(100.0)
            // If a z index or scissoring rect ID is set in an element builder, it will
            // override the default one in `cx`.
            .z_index(OVERLAY_Z_INDEX)
            .build(&mut window_cx);

        let menu_btn = Button::builder()
            .class(CLASS_MENU)
            .icon(MyIcon::Menu)
            .on_select(MyAction::OpenMenu)
            .build(&mut window_cx);
        let menu = DropDownMenu::builder()
            .entries(vec![
                MenuEntry::option_with_right_text("Hello", MenuOption::Hello.right_text(), 0),
                MenuEntry::option_with_right_text("World", MenuOption::World.right_text(), 1),
                MenuEntry::Divider,
                MenuEntry::option_with_right_text("About", MenuOption::About.right_text(), 2),
            ])
            .on_entry_selected(|id| MyAction::MenuItemSelected(MenuOption::ALL[id]))
            .z_index(OVERLAY_Z_INDEX)
            .build(&mut window_cx);

        let tooltip = Tooltip::builder()
            .z_index(OVERLAY_Z_INDEX)
            .build(&mut window_cx);

        let tab_group = TabGroup::new(
            MyTab::ALL
                .map(|t| TabGroupOption::new(Some(format!("{t}")), None, Some(format!("{t}")))),
            current_tab as usize,
            |i| MyAction::TabSelected(MyTab::ALL[i]),
            None,
            IndicatorLinePlacement::Left,
            Align2::CENTER_RIGHT,
            None,
            None,
            &mut window_cx,
        );

        let mut basic_elements = basic_elements::Elements::new(&mut window_cx);
        let mut knobs_and_sliders = knobs_and_sliders::Elements::new(&style, &mut window_cx);

        basic_elements.set_hidden(current_tab != MyTab::BasicElements);
        knobs_and_sliders.set_hidden(current_tab != MyTab::KnobsAndSliders);

        let mut new_self = Self {
            main_window_elements: MainWindowElements {
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
            },
            about_window_elements: None,

            style,
            current_tab,
        };

        new_self.layout_main_window(&mut window_cx);

        window_cx.set_tooltip_actions(
            |info| MyAction::ShowTooltip((info, MAIN_WINDOW)),
            || MyAction::HideTooltip(MAIN_WINDOW),
        );

        Ok(new_self)
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
                if window_id == about_window::ABOUT_WINDOW_ID {
                    let mut about_window_cx = cx.window(about_window::ABOUT_WINDOW_ID).unwrap();

                    self.about_window_elements = Some(about_window::Elements::new(
                        &self.style,
                        &mut about_window_cx,
                    ));
                }
            }
            AppWindowEvent::WindowResized => {
                if window_id == MAIN_WINDOW {
                    self.layout_main_window(&mut cx.main_window());
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
            #[cfg(debug_assertions)]
            dbg!(&action);

            let mut needs_layout = false;

            match action {
                MyAction::BasicElements(action) => {
                    needs_layout = self
                        .main_window_elements
                        .basic_elements
                        .handle_action(action, &mut cx.main_window());
                }
                MyAction::KnobsAndSliders(action) => {
                    needs_layout = self.main_window_elements.knobs_and_sliders.handle_action(
                        action,
                        &self.style,
                        &mut cx.main_window(),
                    );
                }
                MyAction::AboutWindow(action) => {
                    if let Some(mut about_window_cx) = cx.window(about_window::ABOUT_WINDOW_ID) {
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
                    self.main_window_elements.menu.open(None);
                }
                MyAction::ShowTooltip((info, _window_id)) => {
                    self.main_window_elements.tooltip.show(
                        &info.text,
                        info.align,
                        info.element_bounds,
                        &mut cx.res,
                    );
                }
                MyAction::HideTooltip(_window_id) => {
                    self.main_window_elements.tooltip.hide();
                }
                MyAction::TabSelected(tab) => {
                    self.current_tab = tab;
                    self.main_window_elements
                        .tab_group
                        .updated_selected(tab as usize);
                    self.main_window_elements
                        .basic_elements
                        .set_hidden(tab != MyTab::BasicElements);
                    self.main_window_elements
                        .knobs_and_sliders
                        .set_hidden(tab != MyTab::KnobsAndSliders);
                    needs_layout = true;
                }
                MyAction::OpenAboutWindow => {
                    cx.open_window(about_window::ABOUT_WINDOW_ID, about_window::window_config());
                }
            }

            if needs_layout {
                self.layout_main_window(&mut cx.main_window());
            }
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
            && event.modifiers.contains(Modifiers::CONTROL)
            && !event.repeat
        {
            println!("program-wide keyboard shortcut activated: Ctrl+A");
            cx.action_sender.send(MyAction::OpenAboutWindow).unwrap();
        }
    }
}

impl MyApp {
    fn layout_main_window(&mut self, window_cx: &mut WindowContext<MyAction>) {
        let window_size = window_cx.logical_size();

        let el = &mut self.main_window_elements;

        el.top_panel_bg.set_rect(rect(
            0.0,
            0.0,
            window_size.width,
            self.style.top_panel_height,
        ));
        el.top_panel_border.set_rect(rect(
            0.0,
            self.style.top_panel_height - self.style.panel_border_width,
            window_size.width,
            self.style.panel_border_width,
        ));

        let left_panel_width = el.left_panel_resize_handle.current_span();
        el.left_panel_bg.set_rect(rect(
            0.0,
            self.style.top_panel_height,
            left_panel_width,
            window_size.height - self.style.top_panel_height,
        ));
        el.left_panel_border.set_rect(rect(
            left_panel_width - self.style.panel_border_width,
            self.style.top_panel_height,
            self.style.panel_border_width,
            window_size.height - self.style.top_panel_height,
        ));
        el.left_panel_resize_handle.set_layout(ResizeHandleLayout {
            anchor: point(0.0, self.style.top_panel_height),
            length: window_size.height,
        });

        el.menu_btn.layout_aligned(
            point(
                self.style.menu_btn_padding,
                self.style.top_panel_height * 0.5,
            ),
            Align2::CENTER_LEFT,
            window_cx.res,
        );

        el.menu
            .set_position(point(el.menu_btn.min_x(), el.menu_btn.max_y()));

        el.tab_group.layout(
            point(
                0.0,
                self.style.top_panel_height + self.style.tab_group_padding,
            ),
            self.style.tag_group_spacing,
            LayoutDirection::Vertical,
            Some(left_panel_width - self.style.panel_border_width),
            window_cx.res,
        );

        let content_rect = rect(
            left_panel_width,
            self.style.top_panel_height,
            window_size.width - left_panel_width,
            window_size.height - self.style.top_panel_height,
        );

        match self.current_tab {
            MyTab::BasicElements => {
                el.basic_elements
                    .layout(content_rect, &self.style, window_cx);
            }
            MyTab::KnobsAndSliders => {
                el.knobs_and_sliders
                    .layout(content_rect, &self.style, window_cx);
            }
            MyTab::More => {}
        }
    }
}

use std::{any::Any, rc::Rc};

use rootvg::{
    color::RGBA8,
    math::{Point, Rect, Size},
    quad::Radius,
    PrimitiveGroup,
};

use crate::{
    layout::{Padding, SizeType},
    prelude::ElementStyle,
    style::{Background, BorderStyle, QuadStyle, DEFAULT_ACCENT_COLOR, DEFAULT_ACCENT_HOVER_COLOR},
    view::element::RenderContext,
};

use super::virtual_slider::{
    UpdateResult, VirtualSlider, VirtualSliderRenderInfo, VirtualSliderRenderer, VirtualSliderState,
};

#[derive(Debug, Clone)]
pub enum SliderStyle {
    Modern(SliderStyleModern),
}

impl SliderStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        match self {
            Self::Modern(s) => s.states_differ(a, b),
        }
    }
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self::Modern(SliderStyleModern::default())
    }
}

impl ElementStyle for SliderStyle {
    const ID: &'static str = "vs-sldr";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct SliderStyleModern {
    pub back_idle: QuadStyle,
    pub back_hovered: QuadStyle,
    pub back_gesturing: QuadStyle,
    pub back_disabled: QuadStyle,

    pub handle_idle: QuadStyle,
    pub handle_hovered: QuadStyle,
    pub handle_gesturing: QuadStyle,
    pub handle_disabled: QuadStyle,

    pub fill_idle: QuadStyle,
    pub fill_hovered: QuadStyle,
    pub fill_gesturing: QuadStyle,
    pub fill_disabled: QuadStyle,

    pub handle_height: SizeType,
    pub handle_padding: Padding,
    pub fill_padding: Padding,
    pub handle_fill_spacing: f32,
    pub fill_hide_threshold_normal: f64,
    pub fill_mode: SliderFillMode,
}

impl SliderStyleModern {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        let get_styles = |s: VirtualSliderState| -> (&QuadStyle, &QuadStyle, &QuadStyle) {
            match s {
                VirtualSliderState::Idle => (&self.back_idle, &self.handle_idle, &self.fill_idle),
                VirtualSliderState::Hovered => {
                    (&self.back_hovered, &self.handle_hovered, &self.fill_hovered)
                }
                VirtualSliderState::Gesturing => (
                    &self.back_gesturing,
                    &self.handle_gesturing,
                    &self.fill_gesturing,
                ),
                VirtualSliderState::Disabled => (
                    &self.back_disabled,
                    &self.handle_disabled,
                    &self.fill_disabled,
                ),
            }
        };

        let (back_a, handle_a, fill_a) = get_styles(a);
        let (back_b, handle_b, fill_b) = get_styles(b);

        back_a != back_b || handle_a != handle_b || fill_a != fill_b
    }
}

impl Default for SliderStyleModern {
    fn default() -> Self {
        let radius: Radius = 3.0.into();

        let back_idle = QuadStyle {
            bg: Background::Solid(RGBA8::new(25, 25, 25, 255)),
            border: BorderStyle {
                color: RGBA8::new(100, 100, 100, 255),
                width: 1.0,
                radius,
            },
        };

        let back_hovered = QuadStyle {
            border: BorderStyle {
                color: RGBA8::new(135, 135, 135, 255),
                ..back_idle.border
            },
            ..back_idle.clone()
        };

        let handle_idle = QuadStyle {
            bg: Background::Solid(RGBA8::new(208, 208, 208, 255)),
            border: BorderStyle {
                radius,
                color: RGBA8::new(25, 25, 25, 255),
                width: 1.0,
                ..Default::default()
            },
        };

        let fill_idle = QuadStyle {
            bg: Background::Solid(DEFAULT_ACCENT_COLOR),
            border: BorderStyle {
                color: RGBA8::new(105, 105, 105, 255),
                radius,
                ..Default::default()
            },
        };

        let fill_hovered = QuadStyle {
            bg: Background::Solid(DEFAULT_ACCENT_HOVER_COLOR),
            ..fill_idle
        };

        Self {
            back_idle: back_idle.clone(),
            back_hovered: back_hovered.clone(),
            back_gesturing: back_hovered.clone(),
            back_disabled: QuadStyle {
                border: BorderStyle {
                    color: RGBA8::new(65, 65, 65, 255),
                    ..back_idle.border
                },
                ..back_idle
            },

            handle_idle: handle_idle.clone(),
            handle_hovered: QuadStyle {
                bg: Background::Solid(RGBA8::new(215, 215, 215, 255)),
                ..handle_idle
            },
            handle_gesturing: QuadStyle {
                bg: Background::Solid(RGBA8::new(230, 230, 230, 255)),
                ..handle_idle
            },
            handle_disabled: QuadStyle {
                bg: Background::Solid(RGBA8::new(65, 65, 65, 255)),
                ..handle_idle
            },

            fill_idle: fill_idle.clone(),
            fill_hovered: fill_hovered.clone(),
            fill_gesturing: fill_hovered,
            fill_disabled: QuadStyle {
                bg: Background::Solid(RGBA8::new(150, 150, 150, 150)),
                ..fill_idle
            },

            handle_padding: Padding::new(2.0, 2.0, 2.0, 2.0),
            fill_padding: Padding::new(3.0, 3.0, 3.0, 3.0),
            handle_fill_spacing: 1.0,
            handle_height: SizeType::FixedPoints(7.0),
            fill_hide_threshold_normal: 0.005,
            fill_mode: SliderFillMode::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderFillMode {
    #[default]
    CoverHandle,
    AvoidHandle,
}

pub struct SliderRenderer {
    style: Rc<dyn Any>,
}

impl VirtualSliderRenderer for SliderRenderer {
    type Style = SliderStyle;

    fn new(style: Rc<dyn Any>) -> Self {
        Self { style }
    }

    fn style_changed(&mut self, new_style: Rc<dyn Any>) {
        self.style = new_style;
    }

    fn on_state_changed(
        &mut self,
        prev_state: VirtualSliderState,
        new_state: VirtualSliderState,
    ) -> UpdateResult {
        let style = self.style.downcast_ref::<SliderStyle>().unwrap();

        // Only repaint if the appearance is different.
        UpdateResult {
            repaint: style.states_differ(prev_state, new_state),
            animating: false,
        }
    }

    fn render_primitives(
        &mut self,
        info: VirtualSliderRenderInfo<'_>,
        cx: RenderContext<'_>,
        primitives: &mut PrimitiveGroup,
    ) {
        let style = self.style.downcast_ref::<SliderStyle>().unwrap();

        match style {
            SliderStyle::Modern(style) => {
                if info.horizontal {
                    let to_horizontal = |r: Rect| -> Rect {
                        Rect::new(
                            Point::new(cx.bounds_size.width - r.max_y(), r.min_x()),
                            Size::new(r.height(), r.width()),
                        )
                    };

                    let b = ModernStyleBounds::new(
                        Size::new(cx.bounds_size.height, cx.bounds_size.width),
                        style,
                        info,
                    );

                    if let Some(back_style) = b.back {
                        primitives
                            .add(back_style.create_primitive(Rect::from_size(cx.bounds_size)));
                    }

                    if let Some((fill_rect, fill_style)) = b.fill {
                        let fill_rect = to_horizontal(fill_rect);

                        primitives.set_z_index(1);
                        primitives.add(fill_style.create_primitive(fill_rect));
                    }

                    if let Some((handle_rect, handle_style)) = b.handle {
                        let handle_rect = to_horizontal(handle_rect);

                        primitives.set_z_index(2);
                        primitives.add(handle_style.create_primitive(handle_rect));
                    }
                } else {
                    let b = ModernStyleBounds::new(cx.bounds_size, style, info);

                    if let Some(back_style) = b.back {
                        primitives
                            .add(back_style.create_primitive(Rect::from_size(cx.bounds_size)));
                    }

                    if let Some((fill_rect, fill_style)) = b.fill {
                        primitives.set_z_index(1);
                        primitives.add(fill_style.create_primitive(fill_rect));
                    }

                    if let Some((handle_rect, handle_style)) = b.handle {
                        primitives.set_z_index(2);
                        primitives.add(handle_style.create_primitive(handle_rect));
                    }
                }
            }
        }
    }
}

pub type Slider = VirtualSlider<SliderRenderer>;

struct ModernStyleBounds<'a> {
    back: Option<&'a QuadStyle>,
    handle: Option<(Rect, &'a QuadStyle)>,
    fill: Option<(Rect, &'a QuadStyle)>,
}

impl<'a> ModernStyleBounds<'a> {
    fn new(
        bounds_size: Size,
        style: &'a SliderStyleModern,
        info: VirtualSliderRenderInfo<'_>,
    ) -> Self {
        let (back_style, handle_style, fill_style) = match info.state {
            VirtualSliderState::Idle => (&style.back_idle, &style.handle_idle, &style.fill_idle),
            VirtualSliderState::Hovered => (
                &style.back_hovered,
                &style.handle_hovered,
                &style.fill_hovered,
            ),
            VirtualSliderState::Gesturing => (
                &style.back_gesturing,
                &style.handle_gesturing,
                &style.fill_gesturing,
            ),
            VirtualSliderState::Disabled => (
                &style.back_disabled,
                &style.handle_disabled,
                &style.fill_disabled,
            ),
        };

        let handle = if !handle_style.is_transparent() {
            let handle_height = style.handle_height.points(bounds_size.height);

            let handle_span = bounds_size.height
                - style.handle_padding.top
                - style.handle_padding.bottom
                - handle_height;

            let handle_rect = Rect::new(
                Point::new(
                    style.handle_padding.left,
                    bounds_size.height
                        - style.handle_padding.bottom
                        - handle_height
                        - (handle_span * info.normal_value as f32),
                ),
                Size::new(
                    bounds_size.width - style.handle_padding.left - style.handle_padding.right,
                    handle_height,
                ),
            );

            Some((handle_rect, handle_style))
        } else {
            None
        };

        let do_show_fill = if fill_style.is_transparent() {
            false
        } else if info.bipolar {
            !(info.normal_value > 0.5 - style.fill_hide_threshold_normal
                && info.normal_value < 0.5 + style.fill_hide_threshold_normal)
        } else {
            info.normal_value >= style.fill_hide_threshold_normal
        };

        let fill = if do_show_fill {
            let (fill_y, fill_height) = if info.bipolar {
                if info.normal_value > 0.5 {
                    let fill_span = (bounds_size.height * 0.5) - style.fill_padding.top;

                    if let Some((handle_rect, _)) = handle {
                        match style.fill_mode {
                            SliderFillMode::CoverHandle => (
                                handle_rect.min_y(),
                                (bounds_size.height * 0.5) - handle_rect.min_y(),
                            ),
                            SliderFillMode::AvoidHandle => {
                                let mut fill_height =
                                    fill_span * (info.normal_value as f32 - 0.5) * 2.0;
                                let mut fill_y = (bounds_size.height * 0.5) - fill_height;

                                if fill_y < handle_rect.max_y() + style.handle_fill_spacing {
                                    fill_y = handle_rect.max_y() + style.handle_fill_spacing;
                                    fill_height = (bounds_size.height * 0.5) - fill_y;
                                }

                                (fill_y, fill_height)
                            }
                        }
                    } else {
                        let fill_height = fill_span * (info.normal_value as f32 - 0.5) * 2.0;

                        ((bounds_size.height * 0.5) - fill_height, fill_height)
                    }
                } else {
                    let fill_y = bounds_size.height * 0.5;
                    let fill_span = bounds_size.height - style.fill_padding.bottom - fill_y;

                    if let Some((handle_rect, _)) = handle {
                        match style.fill_mode {
                            SliderFillMode::CoverHandle => (fill_y, handle_rect.max_y() - fill_y),
                            SliderFillMode::AvoidHandle => {
                                let mut fill_height =
                                    fill_span * (1.0 - (info.normal_value as f32 * 2.0));

                                if fill_y + fill_height
                                    > handle_rect.min_y() - style.handle_fill_spacing
                                {
                                    fill_height =
                                        handle_rect.min_y() - style.handle_fill_spacing - fill_y;
                                }

                                (fill_y, fill_height)
                            }
                        }
                    } else {
                        let fill_height = fill_span * (1.0 - (info.normal_value as f32 * 2.0));

                        (fill_y, fill_height)
                    }
                }
            } else {
                let fill_span =
                    bounds_size.height - style.fill_padding.top - style.fill_padding.bottom;

                if let Some((handle_rect, _)) = handle {
                    match style.fill_mode {
                        SliderFillMode::CoverHandle => (
                            handle_rect.min_y(),
                            bounds_size.height - style.fill_padding.bottom - handle_rect.min_y(),
                        ),
                        SliderFillMode::AvoidHandle => {
                            let mut fill_height = fill_span * info.normal_value as f32;
                            let mut fill_y =
                                bounds_size.height - style.fill_padding.bottom - fill_height;

                            if fill_y < handle_rect.max_y() + style.handle_fill_spacing {
                                fill_y = handle_rect.max_y() + style.handle_fill_spacing;
                                fill_height =
                                    bounds_size.height - style.fill_padding.bottom - fill_y;
                            }

                            (fill_y, fill_height)
                        }
                    }
                } else {
                    let fill_height = fill_span * info.normal_value as f32;

                    (
                        bounds_size.height - style.fill_padding.bottom - fill_height,
                        fill_height,
                    )
                }
            };

            if fill_height > 0.0 {
                let fill_rect = Rect::new(
                    Point::new(style.fill_padding.left, fill_y),
                    Size::new(
                        bounds_size.width - style.fill_padding.left - style.fill_padding.right,
                        fill_height,
                    ),
                );

                Some((fill_rect, fill_style))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            back: if back_style.is_transparent() {
                None
            } else {
                Some(back_style)
            },
            handle,
            fill,
        }
    }
}

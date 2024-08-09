use std::{any::Any, rc::Rc};

use rootvg::{
    color::{self, RGBA8},
    math::{Point, Rect, Size},
    quad::Radius,
    PrimitiveGroup,
};

use crate::{
    layout::{Padding, SizeType},
    prelude::ElementStyle,
    style::{Background, BorderStyle, DisabledBackground, DisabledColor, QuadStyle},
    view::element::RenderContext,
};

use super::{
    UpdateResult, VirtualSlider, VirtualSliderRenderInfo, VirtualSliderRenderer, VirtualSliderState,
};

#[derive(Debug, Clone)]
pub enum SliderStyle {
    Modern(SliderStyleModern),
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

struct SliderStyleModernState {
    pub back_quad: QuadStyle,
    pub handle_quad: QuadStyle,
    pub fill_quad: QuadStyle,

    pub handle_height: SizeType,
    pub handle_padding: Padding,
    pub fill_padding: Padding,
    pub handle_fill_spacing: f32,
}

#[derive(Debug, Clone)]
pub struct SliderStyleModern {
    pub back_bg: Background,
    pub back_bg_hover: Option<Background>,
    pub back_bg_disabled: DisabledBackground,

    pub back_border_color: RGBA8,
    pub back_border_color_hover: Option<RGBA8>,
    pub back_border_color_disabled: DisabledColor,

    pub back_border_width: f32,
    pub back_border_width_hover: Option<f32>,

    pub back_border_radius: Radius,
    pub back_border_radius_hover: Option<Radius>,

    pub handle_bg: Background,
    pub handle_bg_hover: Option<Background>,
    pub handle_bg_gesturing: Option<Background>,
    pub handle_bg_disabled: DisabledBackground,

    pub handle_border_color: RGBA8,
    pub handle_border_color_hover: Option<RGBA8>,
    pub handle_border_color_gesturing: Option<RGBA8>,
    pub handle_border_color_disabled: DisabledColor,

    pub handle_border_width: f32,
    pub handle_border_width_hover: Option<f32>,

    pub handle_border_radius: Radius,
    pub handle_border_radius_hover: Option<Radius>,

    pub fill_bg: Background,
    pub fill_bg_hover: Option<Background>,
    pub fill_bg_gesturing: Option<Background>,
    pub fill_bg_disabled: DisabledBackground,

    pub handle_height: SizeType,
    pub handle_height_hover: Option<SizeType>,

    pub handle_padding: Padding,
    pub handle_padding_hover: Option<Padding>,

    pub fill_padding: Padding,
    pub fill_padding_hover: Option<Padding>,

    pub handle_fill_spacing: f32,
    pub handle_fill_spacing_hover: Option<f32>,

    pub fill_hide_threshold_normal: f64,
    pub fill_mode: SliderFillMode,
}

impl SliderStyleModern {
    fn state(&self, state: VirtualSliderState) -> SliderStyleModernState {
        match state {
            VirtualSliderState::Gesturing => SliderStyleModernState {
                back_quad: QuadStyle {
                    bg: self.back_bg_hover.unwrap_or(self.back_bg),
                    border: BorderStyle {
                        color: self
                            .back_border_color_hover
                            .unwrap_or(self.back_border_color),
                        width: self
                            .back_border_width_hover
                            .unwrap_or(self.back_border_width),
                        radius: self
                            .back_border_radius_hover
                            .unwrap_or(self.back_border_radius),
                    },
                },
                handle_quad: QuadStyle {
                    bg: self
                        .handle_bg_gesturing
                        .unwrap_or(self.handle_bg_hover.unwrap_or(self.handle_bg)),
                    border: BorderStyle {
                        color: self.handle_border_color_gesturing.unwrap_or(
                            self.handle_border_color_hover
                                .unwrap_or(self.handle_border_color),
                        ),
                        width: self
                            .handle_border_width_hover
                            .unwrap_or(self.handle_border_width),
                        radius: self
                            .handle_border_radius_hover
                            .unwrap_or(self.handle_border_radius),
                    },
                },
                fill_quad: QuadStyle {
                    bg: self
                        .fill_bg_gesturing
                        .unwrap_or(self.fill_bg_hover.unwrap_or(self.fill_bg)),
                    border: BorderStyle {
                        radius: self
                            .back_border_radius_hover
                            .unwrap_or(self.back_border_radius),
                        ..Default::default()
                    },
                },
                handle_height: self.handle_height_hover.unwrap_or(self.handle_height),
                handle_padding: self.handle_padding_hover.unwrap_or(self.handle_padding),
                fill_padding: self.fill_padding_hover.unwrap_or(self.fill_padding),
                handle_fill_spacing: self
                    .handle_fill_spacing_hover
                    .unwrap_or(self.handle_fill_spacing),
            },
            VirtualSliderState::Hovered => SliderStyleModernState {
                back_quad: QuadStyle {
                    bg: self.back_bg_hover.unwrap_or(self.back_bg),
                    border: BorderStyle {
                        color: self
                            .back_border_color_hover
                            .unwrap_or(self.back_border_color),
                        width: self
                            .back_border_width_hover
                            .unwrap_or(self.back_border_width),
                        radius: self
                            .back_border_radius_hover
                            .unwrap_or(self.back_border_radius),
                    },
                },
                handle_quad: QuadStyle {
                    bg: self.handle_bg_hover.unwrap_or(self.handle_bg),
                    border: BorderStyle {
                        color: self
                            .handle_border_color_hover
                            .unwrap_or(self.handle_border_color),
                        width: self
                            .handle_border_width_hover
                            .unwrap_or(self.handle_border_width),
                        radius: self
                            .handle_border_radius_hover
                            .unwrap_or(self.handle_border_radius),
                    },
                },
                fill_quad: QuadStyle {
                    bg: self.fill_bg_hover.unwrap_or(self.fill_bg),
                    border: BorderStyle {
                        radius: self
                            .back_border_radius_hover
                            .unwrap_or(self.back_border_radius),
                        ..Default::default()
                    },
                },
                handle_height: self.handle_height_hover.unwrap_or(self.handle_height),
                handle_padding: self.handle_padding_hover.unwrap_or(self.handle_padding),
                fill_padding: self.fill_padding_hover.unwrap_or(self.fill_padding),
                handle_fill_spacing: self
                    .handle_fill_spacing_hover
                    .unwrap_or(self.handle_fill_spacing),
            },
            VirtualSliderState::Idle => SliderStyleModernState {
                back_quad: QuadStyle {
                    bg: self.back_bg,
                    border: BorderStyle {
                        color: self.back_border_color,
                        width: self.back_border_width,
                        radius: self.back_border_radius,
                    },
                },
                handle_quad: QuadStyle {
                    bg: self.handle_bg,
                    border: BorderStyle {
                        color: self.handle_border_color,
                        width: self.handle_border_width,
                        radius: self.handle_border_radius,
                    },
                },
                fill_quad: QuadStyle {
                    bg: self.fill_bg,
                    border: BorderStyle {
                        radius: self.back_border_radius,
                        ..Default::default()
                    },
                },
                handle_height: self.handle_height,
                handle_padding: self.handle_padding,
                fill_padding: self.fill_padding,
                handle_fill_spacing: self.handle_fill_spacing,
            },
            VirtualSliderState::Disabled => SliderStyleModernState {
                back_quad: QuadStyle {
                    bg: self.back_bg_disabled.get(self.back_bg),
                    border: BorderStyle {
                        color: self.back_border_color_disabled.get(self.back_border_color),
                        width: self.back_border_width,
                        radius: self.back_border_radius,
                    },
                },
                handle_quad: QuadStyle {
                    bg: self.handle_bg_disabled.get(self.handle_bg),
                    border: BorderStyle {
                        color: self
                            .handle_border_color_disabled
                            .get(self.handle_border_color),
                        width: self.handle_border_width,
                        radius: self.handle_border_radius,
                    },
                },
                fill_quad: QuadStyle {
                    bg: self.fill_bg_disabled.get(self.fill_bg),
                    border: BorderStyle {
                        radius: self.back_border_radius,
                        ..Default::default()
                    },
                },
                handle_height: self.handle_height,
                handle_padding: self.handle_padding,
                fill_padding: self.fill_padding,
                handle_fill_spacing: self.handle_fill_spacing,
            },
        }
    }
}

impl Default for SliderStyleModern {
    fn default() -> Self {
        Self {
            back_bg: Background::TRANSPARENT,
            back_bg_hover: None,
            back_bg_disabled: Default::default(),
            back_border_color: color::TRANSPARENT,
            back_border_color_hover: None,
            back_border_color_disabled: Default::default(),
            back_border_width: 0.0,
            back_border_width_hover: None,
            back_border_radius: Radius::default(),
            back_border_radius_hover: None,
            handle_bg: Background::TRANSPARENT,
            handle_bg_hover: None,
            handle_bg_gesturing: None,
            handle_bg_disabled: Default::default(),
            handle_border_color: color::TRANSPARENT,
            handle_border_color_hover: None,
            handle_border_color_gesturing: None,
            handle_border_color_disabled: Default::default(),
            handle_border_width: 0.0,
            handle_border_width_hover: None,
            handle_border_radius: Radius::default(),
            handle_border_radius_hover: None,
            fill_bg: Background::TRANSPARENT,
            fill_bg_hover: None,
            fill_bg_gesturing: None,
            fill_bg_disabled: Default::default(),
            handle_height: SizeType::default(),
            handle_height_hover: None,
            handle_padding: Padding::default(),
            handle_padding_hover: None,
            fill_padding: Padding::default(),
            fill_padding_hover: None,
            handle_fill_spacing: 0.0,
            handle_fill_spacing_hover: None,
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
        _prev_state: VirtualSliderState,
        _new_state: VirtualSliderState,
    ) -> UpdateResult {
        // TODO: Only repaint if the appearance is different.
        UpdateResult {
            repaint: true,
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
                let style_state = style.state(info.state);

                if info.horizontal {
                    let to_horizontal = |r: Rect| -> Rect {
                        Rect::new(
                            Point::new(cx.bounds_size.width - r.max_y(), r.min_x()),
                            Size::new(r.height(), r.width()),
                        )
                    };

                    let r = ModerStyleRects::new(
                        Size::new(cx.bounds_size.height, cx.bounds_size.width),
                        style,
                        &style_state,
                        info,
                    );

                    if r.back {
                        primitives.add(
                            style_state
                                .back_quad
                                .create_primitive(Rect::from_size(cx.bounds_size)),
                        );
                    }

                    if let Some(fill_rect) = r.fill {
                        let fill_rect = to_horizontal(fill_rect);

                        primitives.set_z_index(1);
                        primitives.add(style_state.fill_quad.create_primitive(fill_rect));
                    }

                    if let Some(handle_rect) = r.handle {
                        let handle_rect = to_horizontal(handle_rect);

                        primitives.set_z_index(2);
                        primitives.add(style_state.handle_quad.create_primitive(handle_rect));
                    }
                } else {
                    let r = ModerStyleRects::new(cx.bounds_size, style, &style_state, info);

                    if r.back {
                        primitives.add(
                            style_state
                                .back_quad
                                .create_primitive(Rect::from_size(cx.bounds_size)),
                        );
                    }

                    if let Some(fill_rect) = r.fill {
                        primitives.set_z_index(1);
                        primitives.add(style_state.fill_quad.create_primitive(fill_rect));
                    }

                    if let Some(handle_rect) = r.handle {
                        primitives.set_z_index(2);
                        primitives.add(style_state.handle_quad.create_primitive(handle_rect));
                    }
                }
            }
        }
    }
}

pub type Slider = VirtualSlider<SliderRenderer>;

struct ModerStyleRects {
    back: bool,
    handle: Option<Rect>,
    fill: Option<Rect>,
}

impl ModerStyleRects {
    fn new(
        bounds_size: Size,
        style: &SliderStyleModern,
        style_state: &SliderStyleModernState,
        info: VirtualSliderRenderInfo<'_>,
    ) -> Self {
        let handle = if !style_state.handle_quad.is_transparent() {
            let handle_height = style_state.handle_height.points(bounds_size.height);

            let handle_span = bounds_size.height
                - style_state.handle_padding.top
                - style_state.handle_padding.bottom
                - handle_height;

            let handle_rect = Rect::new(
                Point::new(
                    style_state.handle_padding.left,
                    bounds_size.height
                        - style_state.handle_padding.bottom
                        - handle_height
                        - (handle_span * info.normal_value as f32),
                ),
                Size::new(
                    bounds_size.width
                        - style_state.handle_padding.left
                        - style_state.handle_padding.right,
                    handle_height,
                ),
            );

            Some(handle_rect)
        } else {
            None
        };

        let do_show_fill = if style_state.fill_quad.is_transparent() {
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
                    let fill_span = (bounds_size.height * 0.5) - style_state.fill_padding.top;

                    if let Some(handle_rect) = handle {
                        match style.fill_mode {
                            SliderFillMode::CoverHandle => (
                                handle_rect.min_y(),
                                (bounds_size.height * 0.5) - handle_rect.min_y(),
                            ),
                            SliderFillMode::AvoidHandle => {
                                let mut fill_height =
                                    fill_span * (info.normal_value as f32 - 0.5) * 2.0;
                                let mut fill_y = (bounds_size.height * 0.5) - fill_height;

                                if fill_y < handle_rect.max_y() + style_state.handle_fill_spacing {
                                    fill_y = handle_rect.max_y() + style_state.handle_fill_spacing;
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
                    let fill_span = bounds_size.height - style_state.fill_padding.bottom - fill_y;

                    if let Some(handle_rect) = handle {
                        match style.fill_mode {
                            SliderFillMode::CoverHandle => (fill_y, handle_rect.max_y() - fill_y),
                            SliderFillMode::AvoidHandle => {
                                let mut fill_height =
                                    fill_span * (1.0 - (info.normal_value as f32 * 2.0));

                                if fill_y + fill_height
                                    > handle_rect.min_y() - style_state.handle_fill_spacing
                                {
                                    fill_height = handle_rect.min_y()
                                        - style_state.handle_fill_spacing
                                        - fill_y;
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
                let fill_span = bounds_size.height
                    - style_state.fill_padding.top
                    - style_state.fill_padding.bottom;

                if let Some(handle_rect) = handle {
                    match style.fill_mode {
                        SliderFillMode::CoverHandle => (
                            handle_rect.min_y(),
                            bounds_size.height
                                - style_state.fill_padding.bottom
                                - handle_rect.min_y(),
                        ),
                        SliderFillMode::AvoidHandle => {
                            let mut fill_height = fill_span * info.normal_value as f32;
                            let mut fill_y =
                                bounds_size.height - style_state.fill_padding.bottom - fill_height;

                            if fill_y < handle_rect.max_y() + style_state.handle_fill_spacing {
                                fill_y = handle_rect.max_y() + style_state.handle_fill_spacing;
                                fill_height =
                                    bounds_size.height - style_state.fill_padding.bottom - fill_y;
                            }

                            (fill_y, fill_height)
                        }
                    }
                } else {
                    let fill_height = fill_span * info.normal_value as f32;

                    (
                        bounds_size.height - style_state.fill_padding.bottom - fill_height,
                        fill_height,
                    )
                }
            };

            if fill_height > 0.0 {
                let fill_rect = Rect::new(
                    Point::new(style_state.fill_padding.left, fill_y),
                    Size::new(
                        bounds_size.width
                            - style_state.fill_padding.left
                            - style_state.fill_padding.right,
                        fill_height,
                    ),
                );

                Some(fill_rect)
            } else {
                None
            }
        } else {
            None
        };

        Self {
            back: !style_state.back_quad.is_transparent(),
            handle,
            fill,
        }
    }
}

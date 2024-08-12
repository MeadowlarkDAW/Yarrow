use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::quad::Radius;
use rootvg::text::{CustomGlyphID, FontSystem, TextProperties};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align, Align2, Padding};
use crate::math::{Rect, Size, ZIndex, Vector};
use crate::prelude::{ElementStyle, ResourceCtx};
use crate::style::{Background, BorderStyle, DisabledBackground, DisabledColor, QuadStyle};
use crate::theme::DEFAULT_ICON_SIZE;
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

use super::button::ButtonState;
use super::label::{LabelInner, LabelPaddingInfo, LabelPrimitives, LabelStyle, TextIconLayout};

/// The style of a [`ToggleButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct ToggleButtonStyle {
    /// The properties of the text
    pub text_properties: TextProperties,

    /// The size of the icon in points.
    ///
    /// By default this is set to `20.0`.
    pub icon_size: f32,

    /// The padding around the text.
    ///
    /// By default this has all values set to `0.0`.
    pub text_padding: Padding,
    /// The padding around the icon.
    ///
    /// By default this has all values set to `0.0`.
    pub icon_padding: Padding,
    /// Extra spacing between the text and icon. (This can be negative to
    /// move them closer together).
    ///
    /// By default this set to `0.0`.
    pub text_icon_spacing: f32,

    /// The color of the text
    ///
    /// By default this is set to `color::WHITE`.
    pub text_color: RGBA8,
    /// The color of the text when the button is toggled on
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_on: Option<RGBA8>,
    /// The color of the text when the button is toggled on and hovered.
    ///
    /// If this is `None`, then `text_color_on` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_on_hover: Option<RGBA8>,
    /// The color of the text when the button is toggled on and down.
    ///
    /// If this is `None`, then `text_color_on` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_on_down: Option<RGBA8>,
    pub text_color_on_disabled: DisabledColor,
    /// The color of the text when the button is toggled off and hovered.
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_off_hover: Option<RGBA8>,
    /// The color of the text when the button is toggled off and down.
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_off_down: Option<RGBA8>,
    pub text_color_off_disabled: DisabledColor,

    /// The color of the icon
    ///
    /// If this is `None`, then `itext_color` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color: Option<RGBA8>,
    /// The color of the icon when the button is toggled on
    ///
    /// If this is `None`, then `icon_color` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color_on: Option<RGBA8>,
    /// The color of the icon when the button is toggled on and hovered.
    ///
    /// If this is `None`, then `icon_color_on` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color_on_hover: Option<RGBA8>,
    /// The color of the icon when the button is toggled on and down.
    ///
    /// If this is `None`, then `icon_color_on` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color_on_down: Option<RGBA8>,
    pub icon_color_on_disabled: DisabledColor,
    /// The color of the icon when the button is toggled off and hovered.
    ///
    /// If this is `None`, then `icon_color` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color_off_hover: Option<RGBA8>,
    /// The color of the icon when the button is toggled off and down.
    ///
    /// If this is `None`, then `icon_color` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color_off_down: Option<RGBA8>,
    pub icon_color_off_disabled: DisabledColor,

    /// The background of the background quad.
    pub back_bg: Background,
    /// The background of the background quad when the button is toggled on.
    ///
    /// If this is `None`, then `back_bg` will be used.
    ///
    /// By default this is set to `None`.
    pub back_bg_on: Option<Background>,
    /// The background of the background quad when the button is toggled on and hovered.
    ///
    /// If this is `None`, then `back_bg_on` will be used.
    ///
    /// By default this is set to `None`.
    pub back_bg_on_hover: Option<Background>,
    /// The background of the background quad when the button is toggled on and down.
    ///
    /// If this is `None`, then `back_bg_on` will be used.
    ///
    /// By default this is set to `None`.
    pub back_bg_on_down: Option<Background>,
    pub back_bg_on_disabled: DisabledBackground,
    /// The background of the background quad when the button is toggled off and hovered.
    ///
    /// If this is `None`, then `back_bg` will be used.
    ///
    /// By default this is set to `None`.
    pub back_bg_off_hover: Option<Background>,
    /// The background of the background quad when the button is toggled off and down.
    ///
    /// If this is `None`, then `back_bg` will be used.
    ///
    /// By default this is set to `None`.
    pub back_bg_off_down: Option<Background>,
    pub back_bg_off_disabled: DisabledBackground,

    /// The color of the border on the background quad.
    pub back_border_color: RGBA8,
    /// The color of the border on the background quad when the button is toggled on.
    ///
    /// If this is `None`, then `back_border_color` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_color_on: Option<RGBA8>,
    /// The color of the border on the background quad when the button is toggled on and hovered.
    ///
    /// If this is `None`, then `back_border_color_on` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_color_on_hover: Option<RGBA8>,
    /// The color of the border on the background quad when the button is toggled on and down.
    ///
    /// If this is `None`, then `back_border_color_on` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_color_on_down: Option<RGBA8>,
    pub back_border_color_on_disabled: DisabledColor,
    /// The color of the border on the background quad when the button is toggled off and hovered.
    ///
    /// If this is `None`, then `back_border_color` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_color_off_hover: Option<RGBA8>,
    /// The color of the border on the background quad when the button is toggled off and down.
    ///
    /// If this is `None`, then `back_border_color` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_color_off_down: Option<RGBA8>,
    pub back_border_color_off_disabled: DisabledColor,

    /// The width of the border on the background quad.
    pub back_border_width: f32,
    /// The width of the border on the background quad when the button is toggled on.
    ///
    /// If this is `None`, then `back_border_width` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_width_on: Option<f32>,
    /// The width of the border on the background quad when the button is toggled on and hovered.
    ///
    /// If this is `None`, then `back_border_width_on` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_width_on_hover: Option<f32>,
    /// The width of the border on the background quad when the button is toggled on and down.
    ///
    /// If this is `None`, then `back_border_width_on` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_width_on_down: Option<f32>,
    /// The width of the border on the background quad when the button is toggled off and hovered.
    ///
    /// If this is `None`, then `back_border_width` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_width_off_hover: Option<f32>,
    /// The width of the border on the background quad when the button is toggled off and down.
    ///
    /// If this is `None`, then `back_border_width` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_width_off_down: Option<f32>,

    /// The border radius of the background quad.
    pub back_border_radius: Radius,

    /// The cursor icon to show when the user hovers over this element.
    ///
    /// If this is `None`, then the cursor icon will not be changed.
    ///
    /// By default this is set to `None`.
    pub cursor_icon: Option<CursorIcon>,
}

impl Default for ToggleButtonStyle {
    fn default() -> Self {
        Self {
            text_properties: Default::default(),
            icon_size: DEFAULT_ICON_SIZE,
            text_padding: Padding::default(),
            icon_padding: Padding::default(),
            text_icon_spacing: 0.0,
            text_color: color::WHITE,
            text_color_on: None,
            text_color_on_hover: None,
            text_color_on_down: None,
            text_color_on_disabled: Default::default(),
            text_color_off_hover: None,
            text_color_off_down: None,
            text_color_off_disabled: Default::default(),
            icon_color: None,
            icon_color_on: None,
            icon_color_on_hover: None,
            icon_color_on_down: None,
            icon_color_on_disabled: Default::default(),
            icon_color_off_hover: None,
            icon_color_off_down: None,
            icon_color_off_disabled: Default::default(),
            back_bg: Background::TRANSPARENT,
            back_bg_on: None,
            back_bg_on_hover: None,
            back_bg_on_down: None,
            back_bg_on_disabled: Default::default(),
            back_bg_off_hover: None,
            back_bg_off_down: None,
            back_bg_off_disabled: Default::default(),
            back_border_color: color::TRANSPARENT,
            back_border_color_on: None,
            back_border_color_on_hover: None,
            back_border_color_on_down: None,
            back_border_color_on_disabled: Default::default(),
            back_border_color_off_hover: None,
            back_border_color_off_down: None,
            back_border_color_off_disabled: Default::default(),
            back_border_width: 0.0,
            back_border_width_on: None,
            back_border_width_on_hover: None,
            back_border_width_on_down: None,
            back_border_width_off_hover: None,
            back_border_width_off_down: None,
            back_border_radius: Default::default(),
            cursor_icon: None,
        }
    }
}

impl ToggleButtonStyle {
    pub fn padding_info(&self) -> LabelPaddingInfo {
        LabelPaddingInfo {
            icon_size: self.icon_size,
            text_padding: self.text_padding,
            icon_padding: self.icon_padding,
            text_icon_spacing: self.text_icon_spacing,
        }
    }

    pub fn label_style(&self, state: ButtonState, toggled: bool) -> LabelStyle {
        let (text_color, icon_color, back_quad) = match state {
            ButtonState::Idle => {
                if toggled {
                    let text_color = self.text_color_on.unwrap_or(self.text_color);

                    (
                        text_color,
                        self.icon_color_on
                            .unwrap_or(self.icon_color.unwrap_or(text_color)),
                        QuadStyle {
                            bg: self.back_bg_on.unwrap_or(self.back_bg),
                            border: BorderStyle {
                                color: self.back_border_color_on.unwrap_or(self.back_border_color),
                                width: self.back_border_width_on.unwrap_or(self.back_border_width),
                                radius: self.back_border_radius,
                            },
                        },
                    )
                } else {
                    (
                        self.text_color,
                        self.icon_color.unwrap_or(self.text_color),
                        QuadStyle {
                            bg: self.back_bg,
                            border: BorderStyle {
                                color: self.back_border_color,
                                width: self.back_border_width,
                                radius: self.back_border_radius,
                            },
                        },
                    )
                }
            }
            ButtonState::Hovered => {
                if toggled {
                    let text_color = self
                        .text_color_on_hover
                        .unwrap_or(self.text_color_on.unwrap_or(self.text_color));

                    (
                        text_color,
                        self.icon_color_on_hover.unwrap_or(
                            self.icon_color_on
                                .unwrap_or(self.icon_color.unwrap_or(text_color)),
                        ),
                        QuadStyle {
                            bg: self
                                .back_bg_on_hover
                                .unwrap_or(self.back_bg_on.unwrap_or(self.back_bg)),
                            border: BorderStyle {
                                color: self.back_border_color_on_hover.unwrap_or(
                                    self.back_border_color_on.unwrap_or(self.back_border_color),
                                ),
                                width: self.back_border_width_on_hover.unwrap_or(
                                    self.back_border_width_on.unwrap_or(self.back_border_width),
                                ),
                                radius: self.back_border_radius,
                            },
                        },
                    )
                } else {
                    (
                        self.text_color_off_hover.unwrap_or(self.text_color),
                        self.icon_color_off_hover
                            .unwrap_or(self.icon_color.unwrap_or(self.text_color)),
                        QuadStyle {
                            bg: self.back_bg_off_hover.unwrap_or(self.back_bg),
                            border: BorderStyle {
                                color: self
                                    .back_border_color_off_hover
                                    .unwrap_or(self.back_border_color),
                                width: self
                                    .back_border_width_off_hover
                                    .unwrap_or(self.back_border_width),
                                radius: self.back_border_radius,
                            },
                        },
                    )
                }
            }
            ButtonState::Down => {
                if toggled {
                    let text_color = self
                        .text_color_on_down
                        .unwrap_or(self.text_color_on.unwrap_or(self.text_color));

                    (
                        text_color,
                        self.icon_color_on_down.unwrap_or(
                            self.icon_color_on
                                .unwrap_or(self.icon_color.unwrap_or(text_color)),
                        ),
                        QuadStyle {
                            bg: self
                                .back_bg_on_down
                                .unwrap_or(self.back_bg_on.unwrap_or(self.back_bg)),
                            border: BorderStyle {
                                color: self.back_border_color_on_down.unwrap_or(
                                    self.back_border_color_on.unwrap_or(self.back_border_color),
                                ),
                                width: self.back_border_width_on_down.unwrap_or(
                                    self.back_border_width_on.unwrap_or(self.back_border_width),
                                ),
                                radius: self.back_border_radius,
                            },
                        },
                    )
                } else {
                    (
                        self.text_color_off_down.unwrap_or(self.text_color),
                        self.icon_color_off_down
                            .unwrap_or(self.icon_color.unwrap_or(self.text_color)),
                        QuadStyle {
                            bg: self.back_bg_off_down.unwrap_or(self.back_bg),
                            border: BorderStyle {
                                color: self
                                    .back_border_color_off_down
                                    .unwrap_or(self.back_border_color),
                                width: self
                                    .back_border_width_off_down
                                    .unwrap_or(self.back_border_width),
                                radius: self.back_border_radius,
                            },
                        },
                    )
                }
            }
            ButtonState::Disabled => {
                if toggled {
                    let text_color = self.text_color_on.unwrap_or(self.text_color);

                    (
                        self.text_color_on_disabled.get(text_color),
                        self.icon_color_on_disabled.get(
                            self.icon_color_on
                                .unwrap_or(self.icon_color.unwrap_or(text_color)),
                        ),
                        QuadStyle {
                            bg: self
                                .back_bg_on_disabled
                                .get(self.back_bg_on.unwrap_or(self.back_bg)),
                            border: BorderStyle {
                                color: self.back_border_color_on_disabled.get(
                                    self.back_border_color_on.unwrap_or(self.back_border_color),
                                ),
                                width: self.back_border_width_on.unwrap_or(self.back_border_width),
                                radius: self.back_border_radius,
                            },
                        },
                    )
                } else {
                    (
                        self.text_color_off_disabled.get(self.text_color),
                        self.icon_color_off_disabled
                            .get(self.icon_color.unwrap_or(self.text_color)),
                        QuadStyle {
                            bg: self.back_bg_off_disabled.get(self.back_bg),
                            border: BorderStyle {
                                color: self
                                    .back_border_color_off_disabled
                                    .get(self.back_border_color),
                                width: self.back_border_width_on.unwrap_or(self.back_border_width),
                                radius: self.back_border_radius,
                            },
                        },
                    )
                }
            }
        };

        LabelStyle {
            text_color,
            icon_color: Some(icon_color),
            back_quad,
            text_properties: self.text_properties,
            icon_size: self.icon_size,
            text_padding: self.text_padding,
            icon_padding: self.icon_padding,
            text_icon_spacing: self.text_icon_spacing,
            vertical_align: Align::Center,
        }
    }
}

impl ElementStyle for ToggleButtonStyle {
    const ID: &'static str = "tgbtn";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self {
            text_color: color::BLACK,
            ..Default::default()
        }
    }
}

/// A reusable button struct that can be used by other elements.
pub struct ToggleButtonInner {
    pub toggled: bool,
    state: ButtonState,
    label_inner: LabelInner,
}

impl ToggleButtonInner {
    pub fn new(
        text: Option<impl Into<String>>,
        icon_id: Option<CustomGlyphID>,
        text_offset: Vector,
        icon_offset: Vector,
        icon_scale: f32,
        toggled: bool,
        disabled: bool,
        text_icon_layout: TextIconLayout,
        style: &ToggleButtonStyle,
        font_system: &mut FontSystem,
    ) -> Self {
        let state = ButtonState::new(disabled);

        let label_inner = LabelInner::new(
            text,
            icon_id,
            text_offset,
            icon_offset,
            icon_scale,
            text_icon_layout,
            &style.label_style(state, toggled),
            font_system,
        );

        Self {
            toggled,
            label_inner,
            state,
        }
    }

    /// Returns `true` if the state has changed.
    pub fn set_state(&mut self, state: ButtonState) -> bool {
        if self.state != state {
            self.state = state;
            true
        } else {
            false
        }
    }

    pub fn state(&self) -> ButtonState {
        self.state
    }

    pub fn sync_new_style(&mut self, style: &ToggleButtonStyle, font_system: &mut FontSystem) {
        self.label_inner
            .sync_new_style(&style.label_style(self.state, self.toggled), font_system);
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// If the padded size needs calculated, then the given closure will be used to
    /// extract the padding from the current style (text_padding, icon_padding).
    pub fn desired_size<F: FnOnce() -> LabelPaddingInfo>(&mut self, get_padding: F) -> Size {
        self.label_inner.desired_size(get_padding)
    }

    /// Returns `true` if the text has changed.
    pub fn set_text<F: FnOnce() -> TextProperties>(
        &mut self,
        text: Option<&str>,
        font_system: &mut FontSystem,
        get_text_props: F,
    ) -> bool {
        self.label_inner.set_text(text, font_system, get_text_props)
    }

    pub fn text(&self) -> Option<&str> {
        self.label_inner.text()
    }

    pub fn set_icon(&mut self, icon: Option<CustomGlyphID>) -> bool {
        self.label_inner.set_icon(icon)
    }

    pub fn icon(&self) -> Option<CustomGlyphID> {
        self.label_inner.icon()
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &ToggleButtonStyle,
        font_system: &mut FontSystem,
    ) -> LabelPrimitives {
        self.label_inner.render_primitives(
            bounds,
            &style.label_style(self.state, self.toggled),
            font_system,
        )
    }

    /// An offset that can be used mainly to correct the position of text.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_text_offset(&mut self, offset: Vector) -> bool {
        if self.label_inner.text_offset != offset {
            self.label_inner.text_offset = offset;
            true
        } else {
            false
        }
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_icon_offset(&mut self, offset: Vector) -> bool {
        if self.label_inner.icon_offset != offset {
            self.label_inner.icon_offset = offset;
            true
        } else {
            false
        }
    }

    pub fn text_offset(&self) -> Vector {
        self.label_inner.text_offset
    }

    pub fn icon_offset(&self) -> Vector {
        self.label_inner.icon_offset
    }

    /// Returns `true` if the icon scale has changed.
    pub fn set_icon_scale(&mut self, scale: f32) -> bool {
        if self.label_inner.icon_scale != scale {
            self.label_inner.icon_scale = scale;
            true
        } else {
            false
        }
    }

    pub fn icon_scale(&self) -> f32 {
        self.label_inner.icon_scale
    }
}

pub struct ToggleButtonBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(bool) -> A>>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub toggled: bool,
    pub text: Option<String>,
    pub icon: Option<CustomGlyphID>,
    pub icon_scale: f32,
    pub text_offset: Vector,
    pub icon_offset: Vector,
    pub text_icon_layout: TextIconLayout,
    pub class: Option<&'static str>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> ToggleButtonBuilder<A> {
    pub fn new() -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            toggled: false,
            text: None,
            icon: None,
            icon_scale: 1.0,
            text_offset: Vector::default(),
            icon_offset: Vector::default(),
            text_icon_layout: TextIconLayout::default(),
            class: None,
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> ToggleButton {
        ToggleButtonElement::create(self, cx)
    }

    pub fn on_toggled<F: FnMut(bool) -> A + 'static>(mut self, f: F) -> Self {
        self.action = Some(Box::new(f));
        self
    }

    pub fn tooltip_message(mut self, message: impl Into<String>, align: Align2) -> Self {
        self.tooltip_message = Some(message.into());
        self.tooltip_align = align;
        self
    }

    pub const fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }

    /// The text of the label
    ///
    /// If this method isn't used, then the label will have no text (unless
    /// [`LabelBulder::text_optional`] is used).
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// The icon of the label
    ///
    /// If this method isn't used, then the label will have no icon (unless
    /// [`LabelBulder::icon_optional`] is used).
    pub fn icon(mut self, icon: impl Into<CustomGlyphID>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// The optional text of the label
    ///
    /// If this is set to `None`, then the label will have no text.
    pub fn text_optional(mut self, text: Option<impl Into<String>>) -> Self {
        self.text = text.map(|t| t.into());
        self
    }

    /// The optional icon of the label
    ///
    /// If this is set to `None`, then the label will have no icon.
    pub fn icon_optional(mut self, icon: Option<impl Into<CustomGlyphID>>) -> Self {
        self.icon = icon.map(|i| i.into());
        self
    }

    /// The scaling factor for the icon
    ///
    /// By default this is set to `1.0`.
    pub const fn icon_scale(mut self, scale: f32) -> Self {
        self.icon_scale = scale;
        self
    }

    /// An offset that can be used mainly to correct the position of the text.
    /// This does not effect the position of the background quad.
    ///
    /// By default this is set to an offset of zero.
    pub const fn text_offset(mut self, offset: Vector) -> Self {
        self.text_offset = offset;
        self
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    ///
    /// By default this is set to an offset of zero.
    pub const fn icon_offset(mut self, offset: Vector) -> Self {
        self.icon_offset = offset;
        self
    }

    /// How to layout the text and the icon inside the label's bounds.
    ///
    /// By default this is set to `TextIconLayout::LeftAlignIconThenText`
    pub const fn text_icon_layout(mut self, layout: TextIconLayout) -> Self {
        self.text_icon_layout = layout;
        self
    }

    /// The style class name
    ///
    /// If this method is not used, then the current class from the window context will
    /// be used.
    pub const fn class(mut self, class: &'static str) -> Self {
        self.class = Some(class);
        self
    }

    /// The z index of the element
    ///
    /// If this method is not used, then the current z index from the window context will
    /// be used.
    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = Some(z_index);
        self
    }

    /// The bounding rectangle of the element
    ///
    /// If this method is not used, then the element will have a size and position of
    /// zero and will not be visible until its bounding rectangle is set.
    pub const fn bounding_rect(mut self, rect: Rect) -> Self {
        self.bounding_rect = rect;
        self
    }

    /// Whether or not this element is manually hidden
    ///
    /// By default this is set to `false`.
    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.manually_hidden = hidden;
        self
    }

    /// Whether or not this element is in the disabled state
    ///
    /// By default this is set to `false`.
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The ID of the scissoring rectangle this element belongs to.
    ///
    /// If this method is not used, then the current scissoring rectangle ID from the
    /// window context will be used.
    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

/// A button element with a label.
pub struct ToggleButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(bool) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    cursor_icon: Option<CursorIcon>,
}

impl<A: Clone + 'static> ToggleButtonElement<A> {
    pub fn create(builder: ToggleButtonBuilder<A>, cx: &mut WindowContext<'_, A>) -> ToggleButton {
        let ToggleButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
            toggled,
            text,
            icon,
            icon_scale,
            text_offset,
            icon_offset,
            text_icon_layout,
            class,
            z_index,
            bounding_rect,
            manually_hidden,
            disabled,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);
        let style = cx.res.style_system.get::<ToggleButtonStyle>(class);
        let cursor_icon = style.cursor_icon;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: ToggleButtonInner::new(
                text,
                icon,
                text_offset,
                icon_offset,
                icon_scale,
                toggled,
                disabled,
                text_icon_layout,
                &style,
                &mut cx.res.font_system,
            ),
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                action,
                tooltip_message,
                tooltip_align,
                cursor_icon,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        ToggleButton { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for ToggleButtonElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        match event {
            ElementEvent::CustomStateChanged => {
                cx.request_repaint();
            }
            ElementEvent::StyleChanged => {
                let style = cx.res.style_system.get::<ToggleButtonStyle>(cx.class());
                self.cursor_icon = style.cursor_icon;
            }
            ElementEvent::Pointer(PointerEvent::Moved { just_entered, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                if shared_state.inner.state == ButtonState::Disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if let Some(cursor_icon) = self.cursor_icon {
                    cx.cursor_icon = cursor_icon;
                }

                if just_entered && self.tooltip_message.is_some() {
                    cx.start_hover_timeout();
                }

                if shared_state.inner.state == ButtonState::Idle {
                    let needs_repaint = shared_state.inner.set_state(ButtonState::Hovered);

                    if needs_repaint {
                        cx.request_repaint();
                    }
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                if shared_state.inner.state == ButtonState::Hovered
                    || shared_state.inner.state == ButtonState::Down
                {
                    let needs_repaint = shared_state.inner.set_state(ButtonState::Idle);

                    if needs_repaint {
                        cx.request_repaint();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed { button, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                if button == PointerButton::Primary
                    && (shared_state.inner.state == ButtonState::Idle
                        || shared_state.inner.state == ButtonState::Hovered)
                {
                    shared_state.inner.set_state(ButtonState::Down);
                    shared_state.inner.toggled = !shared_state.inner.toggled;

                    cx.request_repaint();

                    if let Some(action) = &mut self.action {
                        cx.send_action((action)(shared_state.inner.toggled))
                            .unwrap();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                position, button, ..
            }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                if button == PointerButton::Primary
                    && (shared_state.inner.state == ButtonState::Down
                        || shared_state.inner.state == ButtonState::Hovered)
                {
                    let new_state = if cx.is_point_within_visible_bounds(position) {
                        ButtonState::Hovered
                    } else {
                        ButtonState::Idle
                    };

                    let needs_repaint = shared_state.inner.set_state(new_state);

                    if needs_repaint {
                        cx.request_repaint();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { .. }) => {
                if let Some(message) = &self.tooltip_message {
                    cx.show_tooltip(message.clone(), self.tooltip_align, true);
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let label_primitives = shared_state.inner.render_primitives(
            Rect::from_size(cx.bounds_size),
            cx.res.style_system.get(cx.class),
            &mut cx.res.font_system,
        );

        if let Some(quad_primitive) = label_primitives.bg_quad {
            primitives.add(quad_primitive);
        }

        if let Some(p) = label_primitives.text {
            primitives.set_z_index(1);
            primitives.add_text(p);
        }

        if let Some(p) = label_primitives.icon {
            primitives.set_z_index(1);
            primitives.add_text(p);
        }
    }
}

/// A handle to a [`ToggleButtonElement`], a button with a label.
pub struct ToggleButton {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: ToggleButtonInner,
}

impl ToggleButton {
    pub fn builder<A: Clone + 'static>() -> ToggleButtonBuilder<A> {
        ToggleButtonBuilder::new()
    }

    pub fn set_toggled(&mut self, toggled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.toggled != toggled {
            shared_state.inner.toggled = toggled;
            self.el._notify_custom_state_change();
        }
    }

    pub fn toggled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.toggled
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the text and icon.
    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .desired_size(|| {
                res.style_system
                    .get::<ToggleButtonStyle>(self.el.class())
                    .padding_info()
            })
    }

    pub fn set_text(&mut self, text: Option<&str>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_text(text, &mut res.font_system, || {
            res.style_system
                .get::<ToggleButtonStyle>(self.el.class())
                .text_properties
        }) {
            self.el._notify_custom_state_change();
        }
    }

    pub fn set_icon(&mut self, icon: Option<impl Into<CustomGlyphID>>) {
        let icon: Option<CustomGlyphID> = icon.map(|i| i.into());

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon(icon) {
            self.el._notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Option<Ref<'a, str>> {
        Ref::filter_map(RefCell::borrow(&self.shared_state), |s| s.inner.text()).ok()
    }

    pub fn icon(&self) -> Option<CustomGlyphID> {
        RefCell::borrow(&self.shared_state).inner.icon()
    }

    pub fn set_class(&mut self, class: &'static str, res: &mut ResourceCtx) {
        if self.el.class() != class {
            RefCell::borrow_mut(&self.shared_state)
                .inner
                .sync_new_style(res.style_system.get(class), &mut res.font_system);

            self.el._notify_class_change(class);
        }
    }

    /// An offset that can be used mainly to correct the position of the text.
    ///
    /// This does not effect the position of the background quad.
    pub fn set_text_offset(&mut self, offset: Vector) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_text_offset(offset) {
            self.el._notify_custom_state_change();
        }
    }

    /// An offset that can be used mainly to correct the position of the icon.
    ///
    /// This does not effect the position of the background quad.
    pub fn set_icon_offset(&mut self, offset: Vector) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon_offset(offset) {
            self.el._notify_custom_state_change();
        }
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    ///
    /// This does no effect the padded size of the element.
    pub fn set_icon_scale(&mut self, scale: f32) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon_scale(scale) {
            self.el._notify_custom_state_change();
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if disabled && shared_state.inner.state != ButtonState::Disabled {
            shared_state.inner.set_state(ButtonState::Disabled);
            self.el._notify_custom_state_change();
        } else if !disabled && shared_state.inner.state == ButtonState::Disabled {
            shared_state.inner.set_state(ButtonState::Idle);
            self.el._notify_custom_state_change();
        }
    }

    pub fn disabled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.state == ButtonState::Disabled
    }

    pub fn layout(&mut self, origin: Point, res: &mut ResourceCtx) {
        let size = self.desired_size(res);
        self.el.set_rect(Rect::new(origin, size));
    }

    pub fn layout_aligned(&mut self, point: Point, align: Align2, res: &mut ResourceCtx) {
        let size = self.desired_size(res);
        self.el.set_rect(align.align_rect_to_point(point, size));
    }
}

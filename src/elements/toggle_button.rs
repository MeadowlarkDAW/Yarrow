use derive_where::derive_where;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;
use crate::theme::DEFAULT_ICON_SIZE;

use super::button::ButtonState;
use super::label::{LabelInner, LabelPaddingInfo, LabelPrimitives};

/// The style of a [`ToggleButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct ToggleButtonStyle {
    /// The properties of the text
    pub text_properties: TextProperties,

    /// The width and height of the icon in points (if the user hasn't
    /// manually set a size for the icon).
    ///
    /// By default this is set to `20.0`.
    pub default_icon_size: f32,

    /// Whether or not the icon should be snapped to the nearset physical
    /// pixel when rendering.
    ///
    /// By default this is set to `true`.
    pub snap_icon_to_physical_pixel: bool,

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

    /// Additional flags for the quad primitives.
    ///
    /// By default this is set to `QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL`.
    pub quad_flags: QuadFlags,
}

impl Default for ToggleButtonStyle {
    fn default() -> Self {
        Self {
            text_properties: Default::default(),
            default_icon_size: DEFAULT_ICON_SIZE,
            snap_icon_to_physical_pixel: true,
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
            quad_flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        }
    }
}

impl ToggleButtonStyle {
    pub fn padding_info(&self) -> LabelPaddingInfo {
        LabelPaddingInfo {
            default_icon_size: self.default_icon_size,
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
                            flags: self.quad_flags,
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
                            flags: self.quad_flags,
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
                            flags: self.quad_flags,
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
                            flags: self.quad_flags,
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
                            flags: self.quad_flags,
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
                            flags: self.quad_flags,
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
                            flags: self.quad_flags,
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
                            flags: self.quad_flags,
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
            default_icon_size: self.default_icon_size,
            snap_icon_to_physical_pixel: self.snap_icon_to_physical_pixel,
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
        icon_id: Option<IconID>,
        text_offset: Vector,
        icon_offset: Vector,
        icon_size: Option<Size>,
        icon_scale: IconScale,
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
            icon_size,
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

    pub fn set_icon(&mut self, icon: Option<IconID>) -> bool {
        self.label_inner.set_icon(icon)
    }

    pub fn icon(&self) -> Option<IconID> {
        self.label_inner.icon()
    }

    pub fn render(
        &mut self,
        bounds: Rect,
        style: &ToggleButtonStyle,
        font_system: &mut FontSystem,
    ) -> LabelPrimitives {
        self.label_inner.render(
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

    pub fn set_icon_size(&mut self, size: Option<Size>) -> bool {
        self.label_inner.set_icon_size(size)
    }

    pub fn icon_size(&self) -> Option<Size> {
        self.label_inner.icon_size()
    }

    /// Returns `true` if the icon scale has changed.
    pub fn set_icon_scale(&mut self, scale: IconScale) -> bool {
        if self.label_inner.icon_scale != scale {
            self.label_inner.icon_scale = scale;
            true
        } else {
            false
        }
    }

    pub fn icon_scale(&self) -> IconScale {
        self.label_inner.icon_scale
    }

    pub fn disabled(&self) -> bool {
        self.state == ButtonState::Disabled
    }
}

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[element_builder_disabled]
#[element_builder_tooltip]
#[derive_where(Default)]
pub struct ToggleButtonBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(bool) -> A>>,
    pub toggled: bool,
    pub text: Option<String>,
    pub icon: Option<IconID>,
    pub icon_size: Option<Size>,
    pub icon_scale: IconScale,
    pub text_offset: Vector,
    pub icon_offset: Vector,
    pub text_icon_layout: TextIconLayout,
}

impl<A: Clone + 'static> ToggleButtonBuilder<A> {
    pub fn on_toggled<F: FnMut(bool) -> A + 'static>(mut self, f: F) -> Self {
        self.action = Some(Box::new(f));
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
    pub fn icon(mut self, icon: impl Into<IconID>) -> Self {
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
    pub fn icon_optional(mut self, icon: Option<impl Into<IconID>>) -> Self {
        self.icon = icon.map(|i| i.into());
        self
    }

    /// The size of the icon (Overrides the size in the style.)
    pub fn icon_size(mut self, size: impl Into<Option<Size>>) -> Self {
        self.icon_size = size.into();
        self
    }

    /// The scale of an icon, used to make icons look more consistent.
    ///
    /// Note this does not affect any layout, this is just a visual thing.
    pub fn icon_scale(mut self, scale: impl Into<IconScale>) -> Self {
        self.icon_scale = scale.into();
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

    pub fn build(self, window_cx: &mut WindowContext<'_, A>) -> ToggleButton {
        let ToggleButtonBuilder {
            action,
            tooltip_data,
            toggled,
            text,
            icon,
            icon_size,
            icon_scale,
            text_offset,
            icon_offset,
            text_icon_layout,
            class,
            z_index,
            rect,
            manually_hidden,
            disabled,
            scissor_rect,
        } = self;

        let style = window_cx
            .res
            .style_system
            .get::<ToggleButtonStyle>(window_cx.builder_class(class));
        let cursor_icon = style.cursor_icon;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: ToggleButtonInner::new(
                text,
                icon,
                text_offset,
                icon_offset,
                icon_size,
                icon_scale,
                toggled,
                disabled,
                text_icon_layout,
                &style,
                &mut window_cx.res.font_system,
            ),
            tooltip_inner: TooltipInner::new(tooltip_data),
        }));

        let el = ElementBuilder::new(ToggleButtonElement {
            shared_state: Rc::clone(&shared_state),
            action,
            cursor_icon,
        })
        .builder_values(z_index, scissor_rect, class, window_cx)
        .rect(rect)
        .hidden(manually_hidden)
        .flags(ElementFlags::PAINTS | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        .build(window_cx);

        ToggleButton { el, shared_state }
    }
}

/// A button element with a label.
struct ToggleButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(bool) -> A>>,
    cursor_icon: Option<CursorIcon>,
}

impl<A: Clone + 'static> Element<A> for ToggleButtonElement<A> {
    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        shared_state
            .tooltip_inner
            .handle_event(&event, shared_state.inner.disabled(), cx);

        match event {
            ElementEvent::CustomStateChanged => {
                cx.request_repaint();
            }
            ElementEvent::StyleChanged => {
                let style = cx.res.style_system.get::<ToggleButtonStyle>(cx.class());
                self.cursor_icon = style.cursor_icon;
            }
            ElementEvent::Pointer(PointerEvent::Moved { .. }) => {
                if shared_state.inner.state == ButtonState::Disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if let Some(cursor_icon) = self.cursor_icon {
                    cx.cursor_icon = cursor_icon;
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
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render(&mut self, cx: RenderContext, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let label_primitives = shared_state.inner.render(
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
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
#[element_handle_set_tooltip]
pub struct ToggleButton {
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: ToggleButtonInner,
    tooltip_inner: TooltipInner,
}

impl ToggleButton {
    pub fn builder<A: Clone + 'static>() -> ToggleButtonBuilder<A> {
        ToggleButtonBuilder::default()
    }

    /// Set the toggled state of this element.
    ///
    /// Returns `true` if the toggle state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_toggled(&mut self, toggled: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.toggled != toggled {
            shared_state.inner.toggled = toggled;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn toggled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.toggled
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the text and icon.
    ///
    /// This size is automatically cached, so it should be relatively
    /// inexpensive to call.
    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .desired_size(|| {
                res.style_system
                    .get::<ToggleButtonStyle>(self.el.class())
                    .padding_info()
            })
    }

    /// Set the text.
    ///
    /// Returns `true` if the text has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently. However, this method still
    /// involves a string comparison so you may want to call this method
    /// sparingly.
    pub fn set_text(&mut self, text: Option<&str>, res: &mut ResourceCtx) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_text(text, &mut res.font_system, || {
            res.style_system
                .get::<ToggleButtonStyle>(self.el.class())
                .text_properties
        }) {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// Set the icon.
    ///
    /// Returns `true` if the icon has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon(&mut self, icon: Option<impl Into<IconID>>) -> bool {
        let icon: Option<IconID> = icon.map(|i| i.into());

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon(icon) {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn text<'a>(&'a self) -> Option<Ref<'a, str>> {
        Ref::filter_map(RefCell::borrow(&self.shared_state), |s| s.inner.text()).ok()
    }

    pub fn icon(&self) -> Option<IconID> {
        RefCell::borrow(&self.shared_state).inner.icon()
    }

    /// An offset that can be used mainly to correct the position of the text.
    ///
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the offset has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_text_offset(&mut self, offset: Vector) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_text_offset(offset) {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// An offset that can be used mainly to correct the position of the icon.
    ///
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the offset has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon_offset(&mut self, offset: Vector) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon_offset(offset) {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// Set the size of the icon
    ///
    /// If `size` is `None`, then the size specified by the style will be used.
    ///
    /// Returns `true` if the size has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon_size(&mut self, size: impl Into<Option<Size>>) -> bool {
        let size: Option<Size> = size.into();

        if RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_icon_size(size.into())
        {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// The scale of the icon, used to make icons look more consistent.
    ///
    /// Note this does not affect any layout, this is just a visual thing.
    ///
    /// Returns `true` if the scale has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon_scale(&mut self, scale: impl Into<IconScale>) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon_scale(scale.into()) {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// Set the disabled state of this element.
    ///
    /// Returns `true` if the disabled state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_disabled(&mut self, disabled: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if disabled && shared_state.inner.state != ButtonState::Disabled {
            shared_state.inner.set_state(ButtonState::Disabled);
            self.el.notify_custom_state_change();
            true
        } else if !disabled && shared_state.inner.state == ButtonState::Disabled {
            shared_state.inner.set_state(ButtonState::Idle);
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn disabled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.state == ButtonState::Disabled
    }

    /// Layout out the element (with the top-left corner of the bounds set to `origin`).
    ///
    /// Returns `true` if the layout has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn layout(&mut self, origin: Point, res: &mut ResourceCtx) -> bool {
        let size = self.desired_size(res);
        self.el.set_rect(Rect::new(origin, size))
    }

    /// Layout out the element aligned to the given point.
    ///
    /// Returns `true` if the layout has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn layout_aligned(&mut self, point: Point, align: Align2, res: &mut ResourceCtx) -> bool {
        let size = self.desired_size(res);
        self.el.set_rect(align.align_rect_to_point(point, size))
    }
}

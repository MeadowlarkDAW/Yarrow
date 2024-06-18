use keyboard_types::Modifiers;
use rootvg::math::{Point, Vector};

use crate::event::WheelDeltaType;

use super::VirtualSliderConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureState {
    /// The user has just starting gesturing (dragging) this element.
    GestureStarted,
    /// The user is in the process of gesturing (dragging) this element.
    Gesturing,
    /// The user has just finished gesturing (dragging) this element.
    GestureFinished,
}

impl GestureState {
    pub fn is_gesturing(&self) -> bool {
        *self != GestureState::GestureFinished
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamUpdate {
    /// The parameter ID
    pub param_id: u32,
    /// The normalized value in the range `[0.0, 1.0]`
    pub normal_value: f64,
    /// The stepped value (if this parameter is stepped)
    pub stepped_value: Option<u32>,
    /// The current state of gesturing (dragging)
    ///
    /// If this is update is not the result of the user gesturing,
    /// then this will be `None`.
    pub gesture_state: Option<GestureState>,
}

impl ParamUpdate {
    pub fn is_gesturing(&self) -> bool {
        self.gesture_state
            .map(|g| g.is_gesturing())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BeginGestureType {
    Dragging {
        pointer_start_pos: Point,
        start_normal: f64,
    },
    ScrollWheel,
}

#[derive(Clone, Copy)]
struct SteppedParamState {
    value: u32,
    num_steps: u32,
}

/// A reusable "virtual slider" struct that can be used to make
/// elements like knobs and sliders.
pub struct VirtualSliderInner {
    pub param_id: u32,
    pub config: VirtualSliderConfig,
    pub drag_vertically: bool,
    pub scroll_vertically: bool,

    normal_value: f64,
    default_normal: f64,
    continuous_gesture_normal: f64,
    stepped_state: Option<SteppedParamState>,
    current_gesture: Option<BeginGestureType>,
}

impl VirtualSliderInner {
    pub fn new(
        param_id: u32,
        normal_value: f64,
        default_normal: f64,
        num_quantized_steps: Option<u32>,
        config: VirtualSliderConfig,
        drag_vertically: bool,
        scroll_vertically: bool,
    ) -> Self {
        let (normal_value, default_normal, stepped_state) =
            if let Some(num_steps) = num_quantized_steps {
                let stepped_value = param_normal_to_quantized(normal_value, num_steps);

                (
                    param_quantized_to_normal(stepped_value, num_steps),
                    param_snap_normal(default_normal, num_steps),
                    Some(SteppedParamState {
                        value: stepped_value,
                        num_steps,
                    }),
                )
            } else {
                (
                    normal_value.clamp(0.0, 1.0),
                    default_normal.clamp(0.0, 1.0),
                    None,
                )
            };

        Self {
            param_id,
            config,
            drag_vertically,
            scroll_vertically,
            normal_value,
            default_normal,
            stepped_state,
            continuous_gesture_normal: normal_value,
            current_gesture: None,
        }
    }

    pub fn begin_drag_gesture(&mut self, pointer_start_pos: Point) -> Option<ParamUpdate> {
        if self.current_gesture.is_some() {
            None
        } else {
            self.current_gesture = Some(BeginGestureType::Dragging {
                pointer_start_pos,
                start_normal: self.normal_value,
            });

            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: Some(GestureState::GestureStarted),
            })
        }
    }

    pub fn begin_scroll_wheel_gesture(&mut self) -> Option<ParamUpdate> {
        if self.current_gesture.is_some() {
            None
        } else {
            self.current_gesture = Some(BeginGestureType::ScrollWheel);

            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: Some(GestureState::GestureStarted),
            })
        }
    }

    pub fn is_gesturing(&self) -> bool {
        self.current_gesture.is_some()
    }

    pub fn handle_pointer_moved(
        &mut self,
        pointer_pos: Point,
        pointer_delta: Option<Point>,
        modifiers: Modifiers,
    ) -> Option<ParamUpdate> {
        if let Some(BeginGestureType::Dragging {
            pointer_start_pos,
            start_normal,
        }) = &mut self.current_gesture
        {
            let use_pointer_delta = !self.config.disable_pointer_locking && pointer_delta.is_some();

            let apply_fine_adjustment_scalar = if let Some(m) = self.config.fine_adjustment_modifier
            {
                modifiers == m
            } else {
                false
            };

            let (new_gesture_normal, reset_start_pos) = if use_pointer_delta {
                let delta = pointer_delta.unwrap();
                let delta_points = if self.drag_vertically {
                    -delta.y
                } else {
                    delta.x
                };

                let mut delta_normal = delta_points * self.config.drag_scalar;
                if apply_fine_adjustment_scalar {
                    delta_normal *= self.config.fine_adjustment_scalar;
                }

                (
                    self.continuous_gesture_normal + f64::from(delta_normal),
                    true,
                )
            } else if apply_fine_adjustment_scalar {
                let delta_points = if self.drag_vertically {
                    pointer_start_pos.y - pointer_pos.y
                } else {
                    pointer_pos.x - pointer_start_pos.x
                };

                let delta_normal =
                    delta_points * self.config.drag_scalar * self.config.fine_adjustment_scalar;

                (
                    self.continuous_gesture_normal + f64::from(delta_normal),
                    true,
                )
            } else {
                // Use absolute positions instead of deltas for a "better feel".
                let offset = if self.drag_vertically {
                    pointer_start_pos.y - pointer_pos.y
                } else {
                    pointer_pos.x - pointer_start_pos.x
                };

                (
                    *start_normal + f64::from(offset * self.config.drag_scalar),
                    false,
                )
            };

            if reset_start_pos {
                *pointer_start_pos = pointer_pos;
                *start_normal = self.continuous_gesture_normal;
            }

            self.set_new_gesture_normal(new_gesture_normal)
        } else {
            None
        }
    }

    pub fn handle_scroll_wheel(
        &mut self,
        delta_type: WheelDeltaType,
        modifiers: Modifiers,
    ) -> Option<ParamUpdate> {
        if !self.config.use_scroll_wheel {
            return None;
        }

        let apply_fine_adjustment_scalar = if let Some(m) = self.config.fine_adjustment_modifier {
            modifiers == m
        } else {
            false
        };

        let delta = match delta_type {
            WheelDeltaType::Points(points) => points,
            WheelDeltaType::Lines(lines) => lines * self.config.scroll_wheel_points_per_line,
            // Don't handle scrolling by pages.
            WheelDeltaType::Pages(_) => Vector::default(),
        };

        let delta_points = if self.drag_vertically {
            delta.y
        } else {
            delta.x
        };

        if delta_points == 0.0 {
            return None;
        }

        let mut delta_normal = delta_points * self.config.scroll_wheel_scalar;
        if apply_fine_adjustment_scalar {
            delta_normal *= self.config.fine_adjustment_scalar;
        }

        let new_gesture_normal = self.continuous_gesture_normal - f64::from(delta_normal);

        self.set_new_gesture_normal(new_gesture_normal)
    }

    fn set_new_gesture_normal(&mut self, mut new_gesture_normal: f64) -> Option<ParamUpdate> {
        new_gesture_normal = new_gesture_normal.clamp(0.0, 1.0);

        if new_gesture_normal == self.continuous_gesture_normal {
            return None;
        }

        self.continuous_gesture_normal = new_gesture_normal;

        let value_changed = if let Some(stepped_state) = &mut self.stepped_state {
            let new_val = param_normal_to_quantized(new_gesture_normal, stepped_state.num_steps);
            self.normal_value = param_quantized_to_normal(new_val, stepped_state.num_steps);
            let changed = stepped_state.value != new_val;
            stepped_state.value = new_val;
            changed
        } else {
            let changed = self.normal_value != new_gesture_normal;
            self.normal_value = new_gesture_normal;
            changed
        };

        if value_changed {
            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: Some(GestureState::Gesturing),
            })
        } else {
            None
        }
    }

    pub fn finish_gesture(&mut self) -> Option<ParamUpdate> {
        self.current_gesture.take().map(|_| ParamUpdate {
            param_id: self.param_id,
            normal_value: self.normal_value,
            stepped_value: self.stepped_value(),
            gesture_state: Some(GestureState::GestureFinished),
        })
    }

    pub fn reset_to_default(&mut self) -> Option<ParamUpdate> {
        self.continuous_gesture_normal = self.default_normal;

        if let Some(_) = self.current_gesture.take() {
            self.normal_value = self.default_normal;

            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: Some(GestureState::GestureFinished),
            })
        } else if self.normal_value != self.default_normal {
            self.normal_value = self.default_normal;

            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: None,
            })
        } else {
            None
        }
    }

    pub fn stepped_value(&self) -> Option<u32> {
        self.stepped_state.map(|s| s.value)
    }

    pub fn num_quantized_steps(&self) -> Option<u32> {
        self.stepped_state.map(|s| s.num_steps)
    }

    /// Set the normalized value of the virtual slider.
    ///
    /// If the slider is currently gesturing, then the gesture will
    /// be cancelled.
    pub fn set_normal_value(&mut self, new_normal: f64) -> Option<ParamUpdate> {
        let new_normal = if let Some(stepped_state) = &mut self.stepped_state {
            stepped_state.value = param_normal_to_quantized(new_normal, stepped_state.num_steps);

            param_quantized_to_normal(stepped_state.value, stepped_state.num_steps)
        } else {
            new_normal.clamp(0.0, 1.0)
        };

        let state_changed = self.current_gesture.is_some() || self.normal_value != new_normal;

        self.normal_value = new_normal;
        self.continuous_gesture_normal = new_normal;

        let gesture_state = if let Some(_) = self.current_gesture.take() {
            Some(GestureState::GestureFinished)
        } else {
            None
        };

        if state_changed {
            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state,
            })
        } else {
            None
        }
    }

    /// Set the normalized default value of the virtual slider.
    ///
    /// Returns `true` if the default value has changed.
    pub fn set_default_normal(&mut self, new_normal: f64) -> bool {
        let new_normal = self.snap_normal(new_normal);

        let changed = self.default_normal != new_normal;
        self.default_normal = new_normal;
        changed
    }

    pub fn snap_normal(&self, normal: f64) -> f64 {
        if let Some(stepped_state) = self.stepped_state {
            param_snap_normal(normal, stepped_state.num_steps)
        } else {
            normal.clamp(0.0, 1.0)
        }
    }

    pub fn normal_value(&self) -> f64 {
        self.normal_value
    }

    pub fn default_normal(&self) -> f64 {
        self.default_normal
    }
}

pub fn param_quantized_to_normal(value: u32, num_steps: u32) -> f64 {
    if value == 0 || num_steps < 2 {
        0.0
    } else if value >= num_steps - 1 {
        1.0
    } else {
        f64::from(value) / f64::from(num_steps - 1)
    }
}

pub fn param_normal_to_quantized(normal: f64, num_steps: u32) -> u32 {
    if normal <= 0.0 || num_steps < 2 {
        0
    } else if normal >= 1.0 {
        num_steps - 1
    } else {
        (normal * f64::from(num_steps - 1)).round() as u32
    }
}

pub fn param_snap_normal(normal: f64, num_steps: u32) -> f64 {
    param_quantized_to_normal(param_normal_to_quantized(normal, num_steps), num_steps)
}

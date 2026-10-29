use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wps_core::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationCategory {
    Entrance,
    Emphasis,
    Exit,
    MotionPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationEffect {
    Fade,
    FlyIn,
    FlyOut,
    Zoom,
    Wipe,
    Spin,
    Bounce,
    Morph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TriggerType {
    #[default]
    OnClick,
    WithPrevious,
    AfterPrevious,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutCubic,
    Bounce,
    Elastic,
}

impl Easing {
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => t * (2.0 - t),
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let f = 2.0 * t - 2.0;
                    0.5 * f * f * f + 1.0
                }
            }
            Easing::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t2 = t - 1.5 / d1;
                    n1 * t2 * t2 + 0.75
                } else if t < 2.5 / d1 {
                    let t2 = t - 2.25 / d1;
                    n1 * t2 * t2 + 0.9375
                } else {
                    let t2 = t - 2.625 / d1;
                    n1 * t2 * t2 + 0.984375
                }
            }
            Easing::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    - (2.0_f64.powf(10.0 * (t - 1.0))) * ((t - 1.0 - p / 4.0) * (2.0 * std::f64::consts::PI) / p).sin()
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframeAnimation {
    pub id: String,
    pub shape_id: String,
    pub category: AnimationCategory,
    pub effect: AnimationEffect,
    pub trigger: TriggerType,
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub easing: Easing,
}

impl KeyframeAnimation {
    pub fn new(shape_id: &str, category: AnimationCategory, effect: AnimationEffect) -> Self {
        Self {
            id: format!("anim_{}_{:?}", shape_id, effect),
            shape_id: shape_id.to_string(),
            category,
            effect,
            trigger: TriggerType::OnClick,
            duration_ms: 600,
            delay_ms: 0,
            easing: Easing::EaseOutQuad,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimatedTransform {
    pub opacity: f64,
    pub translate: Point,
    pub scale: (f64, f64),
    pub rotate_deg: f64,
    pub clip_fraction: f64,
}

impl Default for AnimatedTransform {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            translate: Point::ZERO,
            scale: (1.0, 1.0),
            rotate_deg: 0.0,
            clip_fraction: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SlideTimeline {
    pub animations: Vec<KeyframeAnimation>,
}

impl SlideTimeline {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    pub fn add_animation(&mut self, anim: KeyframeAnimation) {
        self.animations.push(anim);
    }

    /// Evaluates animated transforms for all shapes at the current animation step and local elapsed time (ms).
    pub fn evaluate_timeline_at(
        &self,
        current_step: usize,
        elapsed_ms: u32,
        stage_bounds: &Rect,
    ) -> HashMap<String, AnimatedTransform> {
        let mut transforms = HashMap::new();

        // Group animations into build steps (each OnClick initiates a new step)
        let mut steps: Vec<Vec<&KeyframeAnimation>> = Vec::new();
        let mut current_step_group: Vec<&KeyframeAnimation> = Vec::new();

        for anim in &self.animations {
            if anim.trigger == TriggerType::OnClick && !current_step_group.is_empty() {
                steps.push(current_step_group);
                current_step_group = Vec::new();
            }
            current_step_group.push(anim);
        }
        if !current_step_group.is_empty() {
            steps.push(current_step_group);
        }

        // Apply animations based on whether their step is past, current, or future
        for (step_idx, group) in steps.iter().enumerate() {
            for anim in group {
                let initial = transforms.entry(anim.shape_id.clone()).or_insert_with(AnimatedTransform::default);

                if step_idx < current_step {
                    // Past step: fully applied state
                    match anim.category {
                        AnimationCategory::Entrance => {
                            initial.opacity = 1.0;
                            initial.scale = (1.0, 1.0);
                            initial.translate = Point::ZERO;
                            initial.rotate_deg = 0.0;
                        }
                        AnimationCategory::Exit => {
                            initial.opacity = 0.0;
                            initial.scale = (0.0, 0.0);
                        }
                        AnimationCategory::Emphasis => {
                            // Returns to neutral or steady state
                        }
                        AnimationCategory::MotionPath => {}
                    }
                } else if step_idx == current_step {
                    // Currently active step: calculate progress
                    let total_delay = anim.delay_ms;
                    let local_time = elapsed_ms.saturating_sub(total_delay);
                    let progress = if anim.duration_ms == 0 {
                        1.0
                    } else {
                        (local_time as f64 / anim.duration_ms as f64).clamp(0.0, 1.0)
                    };
                    let eased = anim.easing.apply(progress);

                    match (anim.category, anim.effect) {
                        (AnimationCategory::Entrance, AnimationEffect::Fade) => {
                            initial.opacity = eased;
                        }
                        (AnimationCategory::Entrance, AnimationEffect::FlyIn) => {
                            initial.opacity = (eased * 1.5).min(1.0);
                            initial.translate = Point::new(0.0, (1.0 - eased) * stage_bounds.height * 0.5);
                        }
                        (AnimationCategory::Entrance, AnimationEffect::Zoom) => {
                            initial.opacity = eased;
                            initial.scale = (eased, eased);
                        }
                        (AnimationCategory::Entrance, AnimationEffect::Spin) => {
                            initial.opacity = eased;
                            initial.rotate_deg = (1.0 - eased) * -360.0;
                        }
                        (AnimationCategory::Entrance, AnimationEffect::Bounce) => {
                            initial.opacity = (eased * 2.0).min(1.0);
                            initial.translate = Point::new(0.0, (1.0 - eased) * -60.0);
                        }
                        (AnimationCategory::Exit, AnimationEffect::Fade) => {
                            initial.opacity = 1.0 - eased;
                        }
                        (AnimationCategory::Exit, AnimationEffect::FlyOut) => {
                            initial.opacity = 1.0 - eased;
                            initial.translate = Point::new(0.0, eased * stage_bounds.height * 0.5);
                        }
                        (AnimationCategory::Exit, AnimationEffect::Zoom) => {
                            initial.opacity = 1.0 - eased;
                            initial.scale = (1.0 - eased, 1.0 - eased);
                        }
                        (AnimationCategory::Emphasis, AnimationEffect::Spin) => {
                            initial.rotate_deg = eased * 360.0;
                        }
                        (AnimationCategory::Emphasis, AnimationEffect::Bounce) => {
                            let bounce_factor = (eased * std::f64::consts::PI * 2.0).sin();
                            initial.translate = Point::new(0.0, -bounce_factor * 20.0);
                        }
                        _ => {
                            initial.opacity = eased;
                        }
                    }
                } else {
                    // Future step: shape with entrance animation is initially invisible!
                    if anim.category == AnimationCategory::Entrance {
                        initial.opacity = 0.0;
                    }
                }
            }
        }

        transforms
    }
}

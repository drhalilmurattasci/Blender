//! Built-in easing functions.

use crate::interpolation::InterpolationMode;
use std::f32::consts::PI;

/// Apply an ease-in curve for the given interpolation mode.
#[inline]
pub fn ease_in(mode: InterpolationMode, t: f32) -> f32 {
    match mode {
        InterpolationMode::Sine => 1.0 - (t * PI * 0.5).cos(),
        InterpolationMode::Quad => t * t,
        InterpolationMode::Cubic => t * t * t,
        InterpolationMode::Quart => t * t * t * t,
        InterpolationMode::Quint => t * t * t * t * t,
        InterpolationMode::Expo => {
            if t <= 0.0 {
                0.0
            } else {
                (2.0_f32).powf(10.0 * (t - 1.0))
            }
        }
        InterpolationMode::Circ => 1.0 - (1.0 - t * t).max(0.0).sqrt(),
        InterpolationMode::Back => {
            let s = 1.70158_f32;
            t * t * ((s + 1.0) * t - s)
        }
        InterpolationMode::Bounce => 1.0 - bounce_out(1.0 - t),
        InterpolationMode::Elastic => elastic_in(t),
        _ => t,
    }
}

/// Apply an ease-out curve for the given interpolation mode.
#[inline]
pub fn ease_out(mode: InterpolationMode, t: f32) -> f32 {
    match mode {
        InterpolationMode::Sine => (t * PI * 0.5).sin(),
        InterpolationMode::Quad => 1.0 - (1.0 - t) * (1.0 - t),
        InterpolationMode::Cubic => 1.0 - (1.0 - t).powi(3),
        InterpolationMode::Quart => 1.0 - (1.0 - t).powi(4),
        InterpolationMode::Quint => 1.0 - (1.0 - t).powi(5),
        InterpolationMode::Expo => {
            if t >= 1.0 {
                1.0
            } else {
                1.0 - (2.0_f32).powf(-10.0 * t)
            }
        }
        InterpolationMode::Circ => (1.0 - (1.0 - t) * (1.0 - t)).max(0.0).sqrt(),
        InterpolationMode::Back => {
            let s = 1.70158_f32;
            let u = t - 1.0;
            u * u * ((s + 1.0) * u + s) + 1.0
        }
        InterpolationMode::Bounce => bounce_out(t),
        InterpolationMode::Elastic => elastic_out(t),
        _ => t,
    }
}

/// Apply an ease-in-out curve for the given interpolation mode.
#[inline]
pub fn ease_in_out(mode: InterpolationMode, t: f32) -> f32 {
    if t < 0.5 {
        ease_in(mode, t * 2.0) * 0.5
    } else {
        0.5 + ease_out(mode, (t - 0.5) * 2.0) * 0.5
    }
}

fn bounce_out(t: f32) -> f32 {
    if t < 1.0 / 2.75 {
        7.5625 * t * t
    } else if t < 2.0 / 2.75 {
        let u = t - 1.5 / 2.75;
        7.5625 * u * u + 0.75
    } else if t < 2.5 / 2.75 {
        let u = t - 2.25 / 2.75;
        7.5625 * u * u + 0.9375
    } else {
        let u = t - 2.625 / 2.75;
        7.5625 * u * u + 0.984375
    }
}

fn elastic_in(t: f32) -> f32 {
    if t <= 0.0 || t >= 1.0 {
        return t;
    }
    let p = 0.3_f32;
    let s = p / 4.0;
    let u = t - 1.0;
    -(2.0_f32.powf(10.0 * u) * ((u - s) * (2.0 * PI) / p).sin())
}

fn elastic_out(t: f32) -> f32 {
    if t <= 0.0 || t >= 1.0 {
        return t;
    }
    let p = 0.3_f32;
    let s = p / 4.0;
    2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() + 1.0
}

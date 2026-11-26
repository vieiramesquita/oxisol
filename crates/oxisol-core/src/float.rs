//! Floating-point abstraction enabling zero-cost `f32` and `f64` execution.

use core::fmt::{Debug, Display};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Numeric trait implemented for `f32` and `f64` across `oxisol-core`.
pub trait Float:
    Copy
    + Clone
    + Default
    + Debug
    + Display
    + PartialEq
    + PartialOrd
    + Send
    + Sync
    + 'static
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
{
    /// Additive identity (`0.0`).
    const ZERO: Self;
    /// Multiplicative identity (`1.0`).
    const ONE: Self;
    /// Constant `2.0`.
    const TWO: Self;
    /// Mathematical constant $\pi$.
    const PI: Self;
    /// Small positive guard epsilon for division stability.
    const EPS_GUARD: Self;
    /// Default mass balance tolerance for conservation audits.
    const MASS_TOL: Self;

    /// Convert from `f64` literal or constant.
    fn from_f64(v: f64) -> Self;
    /// Convert from `f32` literal or value.
    fn from_f32(v: f32) -> Self;
    /// Convert to `f64` for diagnostics and reporting.
    fn to_f64(self) -> f64;
    /// Convert to `f32` for compact output formats.
    fn to_f32(self) -> f32;
    /// Convert from `usize`.
    fn from_usize(v: usize) -> Self;

    /// Absolute value.
    fn abs(self) -> Self;
    /// Maximum of two values (ignoring NaN in favor of the finite operand).
    fn max(self, other: Self) -> Self;
    /// Minimum of two values.
    fn min(self, other: Self) -> Self;
    /// Clamp `self` to `[lo, hi]`.
    #[inline(always)]
    fn clamp_val(self, lo: Self, hi: Self) -> Self {
        self.max(lo).min(hi)
    }
    /// Natural exponential $e^x$.
    fn exp(self) -> Self;
    /// Base-2 exponential $2^x$.
    fn exp2(self) -> Self;
    /// Natural logarithm $\ln(x)$.
    fn ln(self) -> Self;
    /// Square root $\sqrt{x}$.
    fn sqrt(self) -> Self;
    /// Power $x^y$.
    fn powf(self, exp: Self) -> Self;
    /// Sine $\sin(x)$.
    fn sin(self) -> Self;
    /// Cosine $\cos(x)$.
    fn cos(self) -> Self;
    /// Tangent $\tan(x)$.
    fn tan(self) -> Self;
    /// Inverse cosine $\arccos(x)$.
    fn acos(self) -> Self;
    /// Inverse tangent $\arctan(x)$.
    fn atan(self) -> Self;
    /// Floor.
    fn floor(self) -> Self;
    /// Round to nearest integer.
    fn round(self) -> Self;
    /// Returns true if finite (neither NaN nor infinite).
    fn is_finite_val(self) -> bool;
}

impl Float for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const PI: Self = core::f32::consts::PI;
    const EPS_GUARD: Self = 1.0e-7;
    const MASS_TOL: Self = 1.0e-3;

    #[inline(always)]
    fn from_f64(v: f64) -> Self {
        v as f32
    }
    #[inline(always)]
    fn from_f32(v: f32) -> Self {
        v
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }
    #[inline(always)]
    fn from_usize(v: usize) -> Self {
        v as f32
    }
    #[inline(always)]
    fn abs(self) -> Self {
        f32::abs(self)
    }
    #[inline(always)]
    fn max(self, other: Self) -> Self {
        f32::max(self, other)
    }
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        f32::min(self, other)
    }
    #[inline(always)]
    fn exp(self) -> Self {
        f32::exp(self)
    }
    #[inline(always)]
    fn exp2(self) -> Self {
        f32::exp2(self)
    }
    #[inline(always)]
    fn ln(self) -> Self {
        f32::ln(self)
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }
    #[inline(always)]
    fn powf(self, exp: Self) -> Self {
        f32::powf(self, exp)
    }
    #[inline(always)]
    fn sin(self) -> Self {
        f32::sin(self)
    }
    #[inline(always)]
    fn cos(self) -> Self {
        f32::cos(self)
    }
    #[inline(always)]
    fn tan(self) -> Self {
        f32::tan(self)
    }
    #[inline(always)]
    fn acos(self) -> Self {
        f32::acos(self)
    }
    #[inline(always)]
    fn atan(self) -> Self {
        f32::atan(self)
    }
    #[inline(always)]
    fn floor(self) -> Self {
        f32::floor(self)
    }
    #[inline(always)]
    fn round(self) -> Self {
        f32::round(self)
    }
    #[inline(always)]
    fn is_finite_val(self) -> bool {
        self.is_finite()
    }
}

impl Float for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const PI: Self = core::f64::consts::PI;
    const EPS_GUARD: Self = 1.0e-12;
    const MASS_TOL: Self = 1.0e-10;

    #[inline(always)]
    fn from_f64(v: f64) -> Self {
        v
    }
    #[inline(always)]
    fn from_f32(v: f32) -> Self {
        v as f64
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self
    }
    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }
    #[inline(always)]
    fn from_usize(v: usize) -> Self {
        v as f64
    }
    #[inline(always)]
    fn abs(self) -> Self {
        f64::abs(self)
    }
    #[inline(always)]
    fn max(self, other: Self) -> Self {
        f64::max(self, other)
    }
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        f64::min(self, other)
    }
    #[inline(always)]
    fn exp(self) -> Self {
        f64::exp(self)
    }
    #[inline(always)]
    fn exp2(self) -> Self {
        f64::exp2(self)
    }
    #[inline(always)]
    fn ln(self) -> Self {
        f64::ln(self)
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
    #[inline(always)]
    fn powf(self, exp: Self) -> Self {
        f64::powf(self, exp)
    }
    #[inline(always)]
    fn sin(self) -> Self {
        f64::sin(self)
    }
    #[inline(always)]
    fn cos(self) -> Self {
        f64::cos(self)
    }
    #[inline(always)]
    fn tan(self) -> Self {
        f64::tan(self)
    }
    #[inline(always)]
    fn acos(self) -> Self {
        f64::acos(self)
    }
    #[inline(always)]
    fn atan(self) -> Self {
        f64::atan(self)
    }
    #[inline(always)]
    fn floor(self) -> Self {
        f64::floor(self)
    }
    #[inline(always)]
    fn round(self) -> Self {
        f64::round(self)
    }
    #[inline(always)]
    fn is_finite_val(self) -> bool {
        self.is_finite()
    }
}

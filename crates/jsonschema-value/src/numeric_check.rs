#![allow(clippy::must_use_candidate)]

use serde_json::Number;

use super::numeric;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoundOp {
    Lt,
    Lte,
    Gt,
    Gte,
}

impl BoundOp {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Lt),
            1 => Some(Self::Lte),
            2 => Some(Self::Gt),
            3 => Some(Self::Gte),
            _ => None,
        }
    }
}

// An `f64` limit is exact only up to 2^53, so an integer limit keeps its own width and compares
// against the instance in exact arithmetic.
#[derive(Clone, Debug, PartialEq)]
pub enum CompiledBound {
    F64 {
        op: BoundOp,
        limit: f64,
    },
    #[cfg(not(feature = "arbitrary-precision"))]
    I64 {
        op: BoundOp,
        limit: i64,
    },
    #[cfg(not(feature = "arbitrary-precision"))]
    U64 {
        op: BoundOp,
        limit: u64,
    },
    #[cfg(feature = "arbitrary-precision")]
    BigInt {
        op: BoundOp,
        limit: num_bigint::BigInt,
    },
    #[cfg(feature = "arbitrary-precision")]
    BigFrac {
        op: BoundOp,
        limit: fraction::BigFraction,
    },
    /// A limit with more digits than exact arithmetic takes on.
    #[cfg(feature = "arbitrary-precision")]
    Decimal {
        op: BoundOp,
        limit: numeric::bignum::Decimal,
    },
}

// Codegen inlines every multipleOf that fits f64 and only routes arbitrary-precision
// divisors through `compile_multiple_of`, so only the big variants are ever constructed.
#[derive(Clone, Debug, PartialEq)]
pub enum CompiledMultipleOf {
    #[cfg(feature = "arbitrary-precision")]
    BigInt(num_bigint::BigInt),
    #[cfg(feature = "arbitrary-precision")]
    BigFrac(fraction::BigFraction),
    #[cfg(feature = "arbitrary-precision")]
    Decimal(numeric::bignum::Decimal),
    Unsupported,
}

#[inline]
fn check_primitive_bound<T>(op: BoundOp, value: &Number, limit: T) -> bool
where
    T: Copy + num_traits::ToPrimitive,
    u64: num_cmp::NumCmp<T>,
    i64: num_cmp::NumCmp<T>,
    f64: num_cmp::NumCmp<T>,
{
    match op {
        BoundOp::Lt => numeric::lt(value, limit),
        BoundOp::Lte => numeric::le(value, limit),
        BoundOp::Gt => numeric::gt(value, limit),
        BoundOp::Gte => numeric::ge(value, limit),
    }
}

#[cfg(feature = "arbitrary-precision")]
#[inline]
fn infinity_cmp(op: BoundOp, is_negative: bool) -> bool {
    match op {
        BoundOp::Gte | BoundOp::Gt => !is_negative,
        BoundOp::Lte | BoundOp::Lt => is_negative,
    }
}

#[cfg(feature = "arbitrary-precision")]
fn holds(op: BoundOp, ordering: std::cmp::Ordering) -> bool {
    match op {
        BoundOp::Lt => ordering.is_lt(),
        BoundOp::Lte => ordering.is_le(),
        BoundOp::Gt => ordering.is_gt(),
        BoundOp::Gte => ordering.is_ge(),
    }
}

#[cfg(feature = "arbitrary-precision")]
#[inline]
fn check_bigint_bound(op: BoundOp, limit: &num_bigint::BigInt, value: &Number) -> bool {
    if let Some(instance) = numeric::bignum::past_digit_budget(value.as_str()) {
        return holds(
            op,
            instance.cmp(&numeric::bignum::Decimal::of_integer(limit)),
        );
    }
    if let Some(instance_bigint) = numeric::bignum::try_parse_bigint(value) {
        return match op {
            BoundOp::Lt => instance_bigint < *limit,
            BoundOp::Lte => instance_bigint <= *limit,
            BoundOp::Gt => instance_bigint > *limit,
            BoundOp::Gte => instance_bigint >= *limit,
        };
    }

    if let Some(v) = value.as_u64() {
        return match op {
            BoundOp::Lt => numeric::bignum::u64_lt_bigint(v, limit),
            BoundOp::Lte => numeric::bignum::u64_le_bigint(v, limit),
            BoundOp::Gt => numeric::bignum::u64_gt_bigint(v, limit),
            BoundOp::Gte => numeric::bignum::u64_ge_bigint(v, limit),
        };
    }

    if let Some(v) = value.as_i64() {
        return match op {
            BoundOp::Lt => numeric::bignum::i64_lt_bigint(v, limit),
            BoundOp::Lte => numeric::bignum::i64_le_bigint(v, limit),
            BoundOp::Gt => numeric::bignum::i64_gt_bigint(v, limit),
            BoundOp::Gte => numeric::bignum::i64_ge_bigint(v, limit),
        };
    }

    if let Some(v) = value.as_f64() {
        return match op {
            BoundOp::Lt => numeric::bignum::f64_lt_bigint(v, limit),
            BoundOp::Lte => numeric::bignum::f64_le_bigint(v, limit),
            BoundOp::Gt => numeric::bignum::f64_gt_bigint(v, limit),
            BoundOp::Gte => numeric::bignum::f64_ge_bigint(v, limit),
        };
    }

    if let Some(instance_bigfrac) = numeric::bignum::try_parse_bigfraction(value) {
        let limit_frac = numeric::bignum::whole(limit.clone());
        return match op {
            BoundOp::Lt => instance_bigfrac < limit_frac,
            BoundOp::Lte => instance_bigfrac <= limit_frac,
            BoundOp::Gt => instance_bigfrac > limit_frac,
            BoundOp::Gte => instance_bigfrac >= limit_frac,
        };
    }

    infinity_cmp(op, value.as_str().starts_with('-'))
}

#[cfg(feature = "arbitrary-precision")]
#[inline]
fn check_bigfrac_bound(op: BoundOp, limit: &fraction::BigFraction, value: &Number) -> bool {
    if let Some(instance) = numeric::bignum::past_digit_budget(value.as_str()) {
        if let Some(limit) = numeric::bignum::Decimal::of_fraction(limit) {
            return holds(op, instance.cmp(&limit));
        }
    }
    if let Some(instance_bigfrac) = numeric::bignum::try_parse_bigfraction(value) {
        return match op {
            BoundOp::Lt => instance_bigfrac < *limit,
            BoundOp::Lte => instance_bigfrac <= *limit,
            BoundOp::Gt => instance_bigfrac > *limit,
            BoundOp::Gte => instance_bigfrac >= *limit,
        };
    }

    if let Some(v) = value.as_u64() {
        return match op {
            BoundOp::Lt => numeric::bignum::u64_lt_bigfrac(v, limit),
            BoundOp::Lte => numeric::bignum::u64_le_bigfrac(v, limit),
            BoundOp::Gt => numeric::bignum::u64_gt_bigfrac(v, limit),
            BoundOp::Gte => numeric::bignum::u64_ge_bigfrac(v, limit),
        };
    }

    if let Some(v) = value.as_i64() {
        return match op {
            BoundOp::Lt => numeric::bignum::i64_lt_bigfrac(v, limit),
            BoundOp::Lte => numeric::bignum::i64_le_bigfrac(v, limit),
            BoundOp::Gt => numeric::bignum::i64_gt_bigfrac(v, limit),
            BoundOp::Gte => numeric::bignum::i64_ge_bigfrac(v, limit),
        };
    }

    // An integer past i64 is exact as a BigInt, while `f64` rounds it onto limits a fractional
    // digit separates it from, e.g. `-10000000000000000000000000` against the same with `.1`.
    if let Some(instance_bigint) = numeric::bignum::try_parse_bigint(value) {
        let instance_frac = numeric::bignum::whole(instance_bigint);
        return match op {
            BoundOp::Lt => instance_frac < *limit,
            BoundOp::Lte => instance_frac <= *limit,
            BoundOp::Gt => instance_frac > *limit,
            BoundOp::Gte => instance_frac >= *limit,
        };
    }

    if let Some(v) = value.as_f64() {
        return match op {
            BoundOp::Lt => numeric::bignum::f64_lt_bigfrac(v, limit),
            BoundOp::Lte => numeric::bignum::f64_le_bigfrac(v, limit),
            BoundOp::Gt => numeric::bignum::f64_gt_bigfrac(v, limit),
            BoundOp::Gte => numeric::bignum::f64_ge_bigfrac(v, limit),
        };
    }

    // Dynamic BigFraction validators treat this branch as valid because
    // extremely large scientific notation cannot be compared reliably.
    true
}

pub fn compile_bound(op: BoundOp, limit: &Number) -> CompiledBound {
    #[cfg(feature = "arbitrary-precision")]
    {
        if let Some(limit) = numeric::bignum::past_digit_budget(limit.as_str()) {
            return CompiledBound::Decimal { op, limit };
        }
        if let Some(value) = numeric::bignum::try_parse_bigint(limit) {
            return CompiledBound::BigInt { op, limit: value };
        }
        if let Some(value) = numeric::bignum::try_parse_bigfraction(limit) {
            return CompiledBound::BigFrac { op, limit: value };
        }
    }

    #[cfg(not(feature = "arbitrary-precision"))]
    {
        if let Some(value) = limit.as_i64() {
            return CompiledBound::I64 { op, limit: value };
        }
        if let Some(value) = limit.as_u64() {
            return CompiledBound::U64 { op, limit: value };
        }
    }

    if let Some(value) = limit.as_f64() {
        return CompiledBound::F64 { op, limit: value };
    }

    // Past `f64`; reachable without `arbitrary-precision` once another crate in the build enables
    // `serde_json/arbitrary_precision`.
    let limit = if crate::JsonNumber::as_str(limit).starts_with('-') {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };
    CompiledBound::F64 { op, limit }
}

pub fn check_bound(compiled: &CompiledBound, value: &Number) -> bool {
    match compiled {
        CompiledBound::F64 { op, limit } => check_primitive_bound(*op, value, *limit),
        #[cfg(not(feature = "arbitrary-precision"))]
        CompiledBound::I64 { op, limit } => check_primitive_bound(*op, value, *limit),
        #[cfg(not(feature = "arbitrary-precision"))]
        CompiledBound::U64 { op, limit } => check_primitive_bound(*op, value, *limit),
        #[cfg(feature = "arbitrary-precision")]
        CompiledBound::BigInt { op, limit } => check_bigint_bound(*op, limit, value),
        #[cfg(feature = "arbitrary-precision")]
        CompiledBound::BigFrac { op, limit } => check_bigfrac_bound(*op, limit, value),
        #[cfg(feature = "arbitrary-precision")]
        CompiledBound::Decimal { op, limit } => holds(
            *op,
            numeric::bignum::Decimal::parse(value.as_str()).cmp(limit),
        ),
    }
}

#[cfg(feature = "arbitrary-precision")]
pub fn compile_multiple_of(multiple_of: &Number) -> CompiledMultipleOf {
    #[cfg(feature = "arbitrary-precision")]
    {
        if let Some(value) = numeric::bignum::past_digit_budget(multiple_of.as_str()) {
            return CompiledMultipleOf::Decimal(value);
        }
        if let Some(value) = numeric::bignum::try_parse_bigint(multiple_of) {
            return CompiledMultipleOf::BigInt(value);
        }
        if let Some(value) = numeric::bignum::try_parse_bigfraction(multiple_of) {
            return CompiledMultipleOf::BigFrac(value);
        }
    }

    CompiledMultipleOf::Unsupported
}

#[cfg(feature = "arbitrary-precision")]
pub fn check_multiple_of(compiled: &CompiledMultipleOf, value: &Number) -> bool {
    use numeric::bignum::Decimal;

    if let Some(instance) = numeric::bignum::past_digit_budget(value.as_str()) {
        let multiple = match compiled {
            CompiledMultipleOf::BigInt(multiple) => Some(Decimal::of_integer(multiple)),
            CompiledMultipleOf::BigFrac(multiple) => Decimal::of_fraction(multiple),
            CompiledMultipleOf::Decimal(multiple) => Some(multiple.clone()),
            CompiledMultipleOf::Unsupported => return true,
        };
        if let Some(multiple) = multiple {
            return instance.is_multiple_of(&multiple);
        }
    }
    match compiled {
        #[cfg(feature = "arbitrary-precision")]
        CompiledMultipleOf::BigInt(multiple) => {
            use num_bigint::BigInt;

            if let Some(instance_bigint) = numeric::bignum::try_parse_bigint(value) {
                return numeric::bignum::is_multiple_of_bigint(&instance_bigint, multiple);
            }

            if let Some(v) = value.as_u64() {
                let v_bigint = BigInt::from(v);
                return numeric::bignum::is_multiple_of_bigint(&v_bigint, multiple);
            }

            if let Some(v) = value.as_i64() {
                let v_bigint = BigInt::from(v);
                return numeric::bignum::is_multiple_of_bigint(&v_bigint, multiple);
            }

            false
        }
        #[cfg(feature = "arbitrary-precision")]
        CompiledMultipleOf::BigFrac(multiple) => {
            use num_traits::ToPrimitive;

            if let Some(instance_bigfrac) = numeric::bignum::try_parse_bigfraction(value) {
                return numeric::bignum::is_multiple_of_bigfrac(&instance_bigfrac, multiple);
            }

            if let Some(instance_bigint) = numeric::bignum::try_parse_bigint(value) {
                let value_frac = numeric::bignum::whole(instance_bigint);
                return numeric::bignum::is_multiple_of_bigfrac(&value_frac, multiple);
            }

            if let Some(v) = value.as_u64() {
                let value_frac = fraction::BigFraction::from(v);
                return numeric::bignum::is_multiple_of_bigfrac(&value_frac, multiple);
            }

            if let Some(v) = value.as_i64() {
                let value_frac = fraction::BigFraction::from(v);
                return numeric::bignum::is_multiple_of_bigfrac(&value_frac, multiple);
            }

            let multiple_f64 = multiple.to_f64().unwrap_or(f64::INFINITY);
            numeric::is_multiple_of_float(value, multiple_f64)
        }
        CompiledMultipleOf::Decimal(multiple) => {
            Decimal::parse(value.as_str()).is_multiple_of(multiple)
        }
        CompiledMultipleOf::Unsupported => true,
    }
}

/// Which arithmetic the validator uses for a `multipleOf` divisor. A rewrite that moves a divisor
/// between kinds can change verdicts, so only same-kind rewrites preserve membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DivisorKind {
    /// Integer instances take exact integer modulo.
    Whole,
    /// A whole divisor past the exact-modulo guard, where instances go through `f64` remainder.
    WholeLossy,
    /// Every instance goes through rational division.
    Fractional,
}

/// The arithmetic `divisor` selects, mirroring how `multipleOf` compiles.
pub fn divisor_kind(divisor: &Number) -> DivisorKind {
    #[cfg(feature = "arbitrary-precision")]
    {
        if numeric::bignum::exceeds_digit_budget(divisor.as_str()) {
            return DivisorKind::Fractional;
        }
        if numeric::bignum::try_parse_bigint(divisor).is_some() {
            return DivisorKind::Whole;
        }
        if numeric::bignum::try_parse_bigfraction(divisor).is_some() {
            return DivisorKind::Fractional;
        }
    }
    match divisor.as_f64() {
        Some(value) if value.fract() != 0. => DivisorKind::Fractional,
        // `is_multiple_of_integer` keeps exact modulo only while the divisor itself is exact.
        Some(value) if value.abs() <= MAX_SAFE_INTEGER_F64 => DivisorKind::Whole,
        // A divisor with no `f64` form only arises under arbitrary precision, where the exact
        // parses above have already classified it.
        _ => DivisorKind::WholeLossy,
    }
}

const MAX_SAFE_INTEGER_F64: f64 = 9_007_199_254_740_992.0;

/// Whether `value` satisfies `multipleOf: divisor`, deciding it the way the validator does.
pub fn satisfies_multiple_of(divisor: &Number, value: &Number) -> bool {
    #[cfg(feature = "arbitrary-precision")]
    {
        let compiled = compile_multiple_of(divisor);
        if compiled != CompiledMultipleOf::Unsupported {
            return check_multiple_of(&compiled, value);
        }
    }
    // A divisor with no `f64` is one the validator skips, so every instance passes there.
    divisor.as_f64().is_none_or(|limit| {
        if limit.fract() == 0. {
            numeric::is_multiple_of_integer(value, limit)
        } else {
            numeric::is_multiple_of_float(value, limit)
        }
    })
}

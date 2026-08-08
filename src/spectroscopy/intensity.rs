//! An intensity, and the three things that have to travel with it.
//!
//! `docs/decisions/molecules.md` is where this is argued and issue #66 is where
//! it was decided. The sentence the type exists to hold is that an intensity is
//! not a property of a transition alone. A millimetre catalogue quotes it at one
//! reference temperature in one set of units, an infrared database quotes it at
//! another in another, and converting between them needs the partition function
//! at that temperature, which the two sources do not agree about either.
//!
//! So the reference temperature, the unit and the identity of the partition
//! function are parts of a [`Convention`], a [`Convention`] is required to build
//! an [`Intensity`], and there is no route that produces one without all three.
//! Not optional metadata, because a schema storing an intensity as a number and
//! a unit cannot express the disagreement between two sources, and a schema that
//! cannot express a disagreement reports agreement where there is none.
//!
//! The refusal is the other half. Two intensities quoted in different
//! conventions are not comparable, and they become comparable when a conversion
//! into one common convention has been recorded for each of them. The operation
//! refuses and says what is missing rather than converting with an assumed
//! partition function, because an assumed partition function is an invented
//! number that then propagates as data.
//!
//! Two bounds, and neither is softened.
//!
//! A recorded conversion is recorded, not checked. This module holds that
//! somebody wrote down which tabulation was used and what factor came out of it;
//! it does not evaluate a partition function and it does not verify the factor.
//! Whether the factor is the right one is a claim about physics that no reading
//! of this type makes.
//!
//! An Einstein coefficient is not in this family and is not here. It needs no
//! reference temperature, so it is not a convention with parts missing, and the
//! decision record's sentence about which direction converts is about a
//! transition record that does not exist yet. #22 owns that end.

use crate::register::claims::Unit;
use std::fmt;

/// A reference temperature, in kelvin, as the source states it.
///
/// Positive and finite, refused otherwise. Zero is refused with the rest: a
/// partition function at zero kelvin is not what any of these catalogues quotes
/// against, and admitting it would put a value in the record that no conversion
/// can use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReferenceTemperature(f64);

impl ReferenceTemperature {
    pub fn kelvin(value: f64) -> Result<Self, Refused> {
        if !value.is_finite() || value <= 0.0 {
            return Err(Refused::ReferenceTemperatureNotPhysical(value));
        }
        Ok(ReferenceTemperature(value))
    }

    pub fn as_kelvin(self) -> f64 {
        self.0
    }
}

impl fmt::Display for ReferenceTemperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} K", self.0)
    }
}

/// Which tabulation of the partition function was used.
///
/// A reference to a specific tabulation rather than the name of a function. Two
/// sources both saying they used "the partition function" have told you nothing,
/// and that is the case this identity exists to make impossible to write down.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartitionFunctionId(String);

impl PartitionFunctionId {
    pub fn new(text: impl Into<String>) -> Result<Self, Refused> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(Refused::PartitionFunctionNotNamed);
        }
        Ok(PartitionFunctionId(text))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PartitionFunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The convention an intensity is quoted in.
///
/// Three parts, all required together. The fields are private and there is one
/// constructor, so a convention with a part missing is unrepresentable rather
/// than invalid.
#[derive(Debug, Clone, PartialEq)]
pub struct Convention {
    reference_temperature: ReferenceTemperature,
    unit: Unit,
    partition_function: PartitionFunctionId,
}

impl Convention {
    pub fn new(
        reference_temperature: ReferenceTemperature,
        unit: Unit,
        partition_function: PartitionFunctionId,
    ) -> Result<Self, Refused> {
        if unit.as_str().trim().is_empty() {
            return Err(Refused::UnitNotNamed);
        }
        Ok(Convention {
            reference_temperature,
            unit,
            partition_function,
        })
    }

    pub fn reference_temperature(&self) -> ReferenceTemperature {
        self.reference_temperature
    }

    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    pub fn partition_function(&self) -> &PartitionFunctionId {
        &self.partition_function
    }

    /// Which parts of two conventions differ, in a fixed order.
    ///
    /// Empty means the two are one convention. The comparison of the reference
    /// temperature is exact and no tolerance is offered: deciding that 296 K and
    /// 296.5 K are one convention is a judgement about a partition function
    /// rather than about a number, and a tolerance here would make it silently.
    pub fn differences_from(&self, other: &Convention) -> Vec<&'static str> {
        let mut parts = Vec::new();
        if self.reference_temperature != other.reference_temperature {
            parts.push("the reference temperature");
        }
        if self.unit != other.unit {
            parts.push("the unit");
        }
        if self.partition_function != other.partition_function {
            parts.push("the partition function");
        }
        parts
    }
}

impl fmt::Display for Convention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {} under {}",
            self.unit, self.reference_temperature, self.partition_function
        )
    }
}

/// A conversion somebody wrote down, out of one convention and into another.
///
/// The factor is carried rather than computed. What makes this a record and not
/// a calculation is that the tabulation it came out of is named by the target
/// convention, so a reader can go and check it.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedConversion {
    from: Convention,
    into: Convention,
    factor: f64,
}

impl RecordedConversion {
    pub fn new(from: Convention, into: Convention, factor: f64) -> Result<Self, Refused> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(Refused::ConversionFactorNotUsable(factor));
        }
        Ok(RecordedConversion { from, into, factor })
    }

    pub fn out_of(&self) -> &Convention {
        &self.from
    }

    pub fn into_convention(&self) -> &Convention {
        &self.into
    }

    pub fn factor(&self) -> f64 {
        self.factor
    }
}

/// An intensity as one source quotes it, with whatever conversions have been
/// recorded for it.
///
/// The value and the convention arrive together and cannot be separated. The
/// conversions arrive afterwards, one call each, because a conversion is a
/// second act of recording rather than part of reading the value out.
#[derive(Debug, Clone, PartialEq)]
pub struct Intensity {
    value: f64,
    quoted_in: Convention,
    conversions: Vec<RecordedConversion>,
}

impl Intensity {
    pub fn new(value: f64, quoted_in: Convention) -> Result<Self, Refused> {
        if !value.is_finite() {
            return Err(Refused::ValueNotANumber(value));
        }
        Ok(Intensity {
            value,
            quoted_in,
            conversions: Vec::new(),
        })
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn quoted_in(&self) -> &Convention {
        &self.quoted_in
    }

    /// Record a conversion for this intensity.
    ///
    /// Refused where the conversion is out of a convention this intensity is not
    /// quoted in. A conversion out of somewhere else is a true statement about
    /// two other conventions and says nothing about this value, and attaching it
    /// here would make the comparison below succeed on a factor that was never
    /// about this number.
    pub fn record(&mut self, conversion: RecordedConversion) -> Result<(), Refused> {
        let differences = conversion.from.differences_from(&self.quoted_in);
        if !differences.is_empty() {
            return Err(Refused::ConversionFromAnotherConvention {
                quoted_in: self.quoted_in.to_string(),
                conversion_from: conversion.from.to_string(),
            });
        }
        self.conversions.push(conversion);
        Ok(())
    }

    /// This value expressed in one convention.
    ///
    /// The identity case costs nothing and is not a conversion: a value already
    /// quoted in the target convention is returned as it was read. Anything else
    /// needs a recorded conversion and is refused without one.
    pub fn in_convention(&self, target: &Convention) -> Result<f64, Refused> {
        if self.quoted_in.differences_from(target).is_empty() {
            return Ok(self.value);
        }
        match self
            .conversions
            .iter()
            .find(|conversion| conversion.into.differences_from(target).is_empty())
        {
            Some(conversion) => Ok(self.value * conversion.factor),
            None => Err(Refused::ConversionNotRecorded {
                quoted_in: self.quoted_in.to_string(),
                wanted: target.to_string(),
                differing: self.quoted_in.differences_from(target).join(" and "),
            }),
        }
    }

    /// Every convention this value can be stated in without inventing anything.
    pub fn reachable_conventions(&self) -> Vec<&Convention> {
        let mut reachable = vec![&self.quoted_in];
        reachable.extend(self.conversions.iter().map(|conversion| &conversion.into));
        reachable
    }
}

/// Two intensities stated in one convention, with that convention named.
///
/// The convention is part of the answer rather than something the caller is
/// trusted to remember, because the pair of numbers is the thing that gets
/// copied somewhere else.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub at: Convention,
    pub first: f64,
    pub second: f64,
}

/// Two intensities, in one convention, or the reason they are not comparable.
///
/// Where the two are already in one convention that is the answer. Otherwise a
/// convention both can reach is looked for among what has been recorded, and the
/// absence of one is a refusal naming what differs.
pub fn compare(first: &Intensity, second: &Intensity) -> Result<Comparison, Refused> {
    for candidate in first.reachable_conventions() {
        if let (Ok(a), Ok(b)) = (
            first.in_convention(candidate),
            second.in_convention(candidate),
        ) {
            return Ok(Comparison {
                at: candidate.clone(),
                first: a,
                second: b,
            });
        }
    }
    let differing = first
        .quoted_in
        .differences_from(&second.quoted_in)
        .join(" and ");
    Err(Refused::NotComparable {
        first: first.quoted_in.to_string(),
        second: second.quoted_in.to_string(),
        differing,
    })
}

/// Why an intensity, a convention, a conversion or a comparison was refused.
#[derive(Debug, Clone, PartialEq)]
pub enum Refused {
    /// A reference temperature that is not a temperature a partition function
    /// could be tabulated at.
    ReferenceTemperatureNotPhysical(f64),
    /// A convention with no partition function named.
    PartitionFunctionNotNamed,
    /// A convention with no unit named.
    UnitNotNamed,
    /// An intensity whose value is not a number.
    ValueNotANumber(f64),
    /// A conversion factor that cannot be applied.
    ConversionFactorNotUsable(f64),
    /// A conversion recorded against an intensity it is not about.
    ConversionFromAnotherConvention {
        quoted_in: String,
        conversion_from: String,
    },
    /// A value asked for in a convention no recorded conversion reaches.
    ConversionNotRecorded {
        quoted_in: String,
        wanted: String,
        differing: String,
    },
    /// Two intensities in two conventions with no conversion recorded into one
    /// they share.
    NotComparable {
        first: String,
        second: String,
        differing: String,
    },
}

impl Refused {
    /// A stable name for the constraint this refusal is about.
    ///
    /// The match is exhaustive, so adding a variant does not compile until it is
    /// named here, and naming it here does not pass
    /// `tests/intensity_conventions.rs` until a case trips it.
    pub fn constraint(&self) -> &'static str {
        match self {
            Refused::ReferenceTemperatureNotPhysical(_) => {
                "a reference temperature no partition function could be tabulated at"
            }
            Refused::PartitionFunctionNotNamed => "a convention naming no partition function",
            Refused::UnitNotNamed => "a convention naming no unit",
            Refused::ValueNotANumber(_) => "an intensity whose value is not a number",
            Refused::ConversionFactorNotUsable(_) => "a conversion factor that cannot be applied",
            Refused::ConversionFromAnotherConvention { .. } => {
                "a conversion recorded against an intensity it is not about"
            }
            Refused::ConversionNotRecorded { .. } => {
                "a value asked for in a convention no recorded conversion reaches"
            }
            Refused::NotComparable { .. } => {
                "two intensities across conventions with no conversion recorded"
            }
        }
    }

    /// Every constraint this module can refuse.
    ///
    /// A list, because Rust cannot enumerate an enum's variants. What stops it
    /// drifting is the exhaustive match above and the coverage test that reads
    /// both.
    pub const CONSTRAINTS: [&'static str; 8] = [
        "a reference temperature no partition function could be tabulated at",
        "a convention naming no partition function",
        "a convention naming no unit",
        "an intensity whose value is not a number",
        "a conversion factor that cannot be applied",
        "a conversion recorded against an intensity it is not about",
        "a value asked for in a convention no recorded conversion reaches",
        "two intensities across conventions with no conversion recorded",
    ];
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::ReferenceTemperatureNotPhysical(value) => write!(
                f,
                "{value} is not a reference temperature a partition function is tabulated at"
            ),
            Refused::PartitionFunctionNotNamed => write!(
                f,
                "a convention states no partition function, so no conversion out of it is checkable"
            ),
            Refused::UnitNotNamed => write!(f, "a convention states no unit"),
            Refused::ValueNotANumber(value) => write!(f, "{value} is not an intensity"),
            Refused::ConversionFactorNotUsable(factor) => {
                write!(f, "{factor} cannot be applied as a conversion factor")
            }
            Refused::ConversionFromAnotherConvention {
                quoted_in,
                conversion_from,
            } => write!(
                f,
                "this intensity is quoted in {quoted_in} and the conversion is out of \
                 {conversion_from}, so it says nothing about this value"
            ),
            Refused::ConversionNotRecorded {
                quoted_in,
                wanted,
                differing,
            } => write!(
                f,
                "quoted in {quoted_in}, asked for {wanted}, and no conversion is recorded; \
                 {differing} differs"
            ),
            Refused::NotComparable {
                first,
                second,
                differing,
            } => write!(
                f,
                "{first} and {second} are not comparable until a conversion into one shared \
                 convention is recorded for each; {differing} differs"
            ),
        }
    }
}

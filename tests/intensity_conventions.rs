//! Every constraint the intensity record can refuse has a case that trips
//! exactly it, and a neighbour one change away that it does not trip.
//!
//! The shape is the one `tests/schema_validation.rs` uses over the register's own
//! constraints, and it is repeated here rather than shared because the two read
//! different types and a shared harness over two `Refused` enumerations would be
//! the abstraction that hides which of them a case reached.
//!
//! What each case is about is one violation. A case violating three things at
//! once cannot tell you which of the three the guard catches, and one of them can
//! stop working without anybody noticing.
//!
//! The last three tests are what keep this file true as the record grows.
//! Adding a variant to `Refused` does not compile until `constraint()` names it,
//! naming it does not pass until a case reaches it, and a case naming something
//! the module no longer declares is refused in the other direction.
//!
//! What this file does not cover is printed by every run and is not left to be
//! discovered.

use linienbuch::register::claims::Unit;
use linienbuch::spectroscopy::intensity::{
    Comparison, Convention, Intensity, PartitionFunctionId, RecordedConversion,
    ReferenceTemperature, Refused, compare,
};
use std::collections::BTreeSet;

/// The millimetre catalogue's convention: an intensity at 300 K.
fn millimetre() -> Convention {
    Convention::new(
        ReferenceTemperature::kelvin(300.0).expect("300 K is a temperature"),
        Unit::new("nm2 MHz"),
        PartitionFunctionId::new("cdms-2024-partition-table").expect("a named tabulation"),
    )
    .expect("all three parts")
}

/// The infrared database's convention: the same physical quantity at 296 K, in
/// its own units, under its own tabulation. Every part differs.
fn infrared() -> Convention {
    Convention::new(
        ReferenceTemperature::kelvin(296.0).expect("296 K is a temperature"),
        Unit::new("cm-1/(molecule cm-2)"),
        PartitionFunctionId::new("hitran-2020-tips").expect("a named tabulation"),
    )
    .expect("all three parts")
}

/// A third convention, which is where two sources have to meet if they are going
/// to be compared without either of them being taken as the reference.
fn common() -> Convention {
    Convention::new(
        ReferenceTemperature::kelvin(296.0).expect("296 K is a temperature"),
        Unit::new("cm-1/(molecule cm-2)"),
        PartitionFunctionId::new("cdms-2024-partition-table").expect("a named tabulation"),
    )
    .expect("all three parts")
}

fn quoted(value: f64, convention: Convention) -> Intensity {
    Intensity::new(value, convention).expect("a finite value")
}

/// One case: what it does, and what it is expected to trip.
struct Case {
    name: &'static str,
    /// Run the case and return every refusal it produced.
    run: fn() -> Vec<Refused>,
    /// The same case with the one violation repaired.
    neighbour: fn() -> Vec<Refused>,
    /// The one constraint this case is about.
    constraint: &'static str,
}

fn a_reference_temperature_at_absolute_zero() -> Vec<Refused> {
    ReferenceTemperature::kelvin(0.0)
        .err()
        .into_iter()
        .collect()
}

fn a_reference_temperature_a_table_is_quoted_at() -> Vec<Refused> {
    ReferenceTemperature::kelvin(296.0)
        .err()
        .into_iter()
        .collect()
}

fn a_convention_that_names_no_partition_function() -> Vec<Refused> {
    PartitionFunctionId::new("   ").err().into_iter().collect()
}

fn a_convention_that_names_one() -> Vec<Refused> {
    PartitionFunctionId::new("hitran-2020-tips")
        .err()
        .into_iter()
        .collect()
}

fn a_convention_that_names_no_unit() -> Vec<Refused> {
    Convention::new(
        ReferenceTemperature::kelvin(296.0).expect("296 K is a temperature"),
        Unit::new("  "),
        PartitionFunctionId::new("hitran-2020-tips").expect("a named tabulation"),
    )
    .err()
    .into_iter()
    .collect()
}

fn a_convention_that_names_its_unit() -> Vec<Refused> {
    Convention::new(
        ReferenceTemperature::kelvin(296.0).expect("296 K is a temperature"),
        Unit::new("cm-1/(molecule cm-2)"),
        PartitionFunctionId::new("hitran-2020-tips").expect("a named tabulation"),
    )
    .err()
    .into_iter()
    .collect()
}

fn an_intensity_that_is_not_a_number() -> Vec<Refused> {
    Intensity::new(f64::NAN, infrared())
        .err()
        .into_iter()
        .collect()
}

fn an_intensity_that_is_a_number() -> Vec<Refused> {
    Intensity::new(3.4e-21, infrared())
        .err()
        .into_iter()
        .collect()
}

fn a_conversion_factor_of_zero() -> Vec<Refused> {
    RecordedConversion::new(millimetre(), common(), 0.0)
        .err()
        .into_iter()
        .collect()
}

fn a_conversion_factor_that_applies() -> Vec<Refused> {
    RecordedConversion::new(millimetre(), common(), 4.16e-19)
        .err()
        .into_iter()
        .collect()
}

fn a_conversion_out_of_a_convention_this_value_is_not_in() -> Vec<Refused> {
    let mut intensity = quoted(1.2e-4, millimetre());
    let elsewhere = RecordedConversion::new(infrared(), common(), 2.0).expect("a usable factor");
    intensity.record(elsewhere).err().into_iter().collect()
}

fn a_conversion_out_of_the_convention_this_value_is_in() -> Vec<Refused> {
    let mut intensity = quoted(1.2e-4, millimetre());
    let its_own = RecordedConversion::new(millimetre(), common(), 2.0).expect("a usable factor");
    intensity.record(its_own).err().into_iter().collect()
}

fn a_value_asked_for_where_nothing_reaches() -> Vec<Refused> {
    quoted(1.2e-4, millimetre())
        .in_convention(&common())
        .err()
        .into_iter()
        .collect()
}

fn a_value_asked_for_where_a_conversion_reaches() -> Vec<Refused> {
    let mut intensity = quoted(1.2e-4, millimetre());
    intensity
        .record(RecordedConversion::new(millimetre(), common(), 2.0).expect("a usable factor"))
        .expect("its own convention");
    intensity
        .in_convention(&common())
        .err()
        .into_iter()
        .collect()
}

fn two_intensities_with_no_conversion_recorded() -> Vec<Refused> {
    compare(&quoted(1.2e-4, millimetre()), &quoted(3.4e-21, infrared()))
        .err()
        .into_iter()
        .collect()
}

fn two_intensities_with_both_conversions_recorded() -> Vec<Refused> {
    let (first, second) = both_converted_into_the_common_convention();
    compare(&first, &second).err().into_iter().collect()
}

/// The pair the Done-when of #66 is about: one quantity, two source conventions,
/// a conversion into a third convention recorded for each.
fn both_converted_into_the_common_convention() -> (Intensity, Intensity) {
    let mut first = quoted(1.2e-4, millimetre());
    first
        .record(RecordedConversion::new(millimetre(), common(), 4.16e-19).expect("a usable factor"))
        .expect("its own convention");
    let mut second = quoted(3.4e-21, infrared());
    second
        .record(RecordedConversion::new(infrared(), common(), 1.07).expect("a usable factor"))
        .expect("its own convention");
    (first, second)
}

const CASES: [Case; 8] = [
    Case {
        name: "a reference temperature at absolute zero",
        run: a_reference_temperature_at_absolute_zero,
        neighbour: a_reference_temperature_a_table_is_quoted_at,
        constraint: "a reference temperature no partition function could be tabulated at",
    },
    Case {
        name: "a convention that names no partition function",
        run: a_convention_that_names_no_partition_function,
        neighbour: a_convention_that_names_one,
        constraint: "a convention naming no partition function",
    },
    Case {
        name: "a convention that names no unit",
        run: a_convention_that_names_no_unit,
        neighbour: a_convention_that_names_its_unit,
        constraint: "a convention naming no unit",
    },
    Case {
        name: "an intensity that is not a number",
        run: an_intensity_that_is_not_a_number,
        neighbour: an_intensity_that_is_a_number,
        constraint: "an intensity whose value is not a number",
    },
    Case {
        name: "a conversion factor of zero",
        run: a_conversion_factor_of_zero,
        neighbour: a_conversion_factor_that_applies,
        constraint: "a conversion factor that cannot be applied",
    },
    Case {
        name: "a conversion out of a convention this value is not in",
        run: a_conversion_out_of_a_convention_this_value_is_not_in,
        neighbour: a_conversion_out_of_the_convention_this_value_is_in,
        constraint: "a conversion recorded against an intensity it is not about",
    },
    Case {
        name: "a value asked for where nothing reaches",
        run: a_value_asked_for_where_nothing_reaches,
        neighbour: a_value_asked_for_where_a_conversion_reaches,
        constraint: "a value asked for in a convention no recorded conversion reaches",
    },
    Case {
        name: "two intensities with no conversion recorded",
        run: two_intensities_with_no_conversion_recorded,
        neighbour: two_intensities_with_both_conversions_recorded,
        constraint: "two intensities across conventions with no conversion recorded",
    },
];

#[test]
fn every_case_refuses_exactly_the_constraint_it_is_about() {
    for case in &CASES {
        let refused: Vec<&'static str> = (case.run)().iter().map(Refused::constraint).collect();
        assert_eq!(
            refused,
            vec![case.constraint],
            "the case {:?} must refuse its own constraint and nothing else",
            case.name
        );
    }
}

#[test]
fn every_neighbour_is_refused_by_nothing() {
    for case in &CASES {
        let refused: Vec<&'static str> =
            (case.neighbour)().iter().map(Refused::constraint).collect();
        assert!(
            refused.is_empty(),
            "the neighbour of {:?} is one change away and must pass, got {refused:?}",
            case.name
        );
    }
}

/// The one that keeps this file honest as the record grows.
#[test]
fn every_constraint_this_record_can_refuse_has_a_case() {
    let declared: BTreeSet<&'static str> = Refused::CONSTRAINTS.into_iter().collect();
    let covered: BTreeSet<&'static str> = CASES
        .iter()
        .flat_map(|case| (case.run)())
        .map(|refusal| refusal.constraint())
        .collect();

    let uncovered: Vec<&&'static str> = declared.difference(&covered).collect();
    assert!(
        uncovered.is_empty(),
        "constraints this record can refuse and no case reaches: {uncovered:?}"
    );

    let unknown: Vec<&&'static str> = covered.difference(&declared).collect();
    assert!(
        unknown.is_empty(),
        "cases refusing something this record does not declare: {unknown:?}"
    );

    println!(
        "intensity conventions: {} constraint(s) declared, {} case(s), each with a neighbour",
        declared.len(),
        CASES.len()
    );
}

#[test]
fn the_declared_list_holds_what_the_match_returns() {
    let declared: BTreeSet<&'static str> = Refused::CONSTRAINTS.into_iter().collect();
    assert_eq!(
        declared.len(),
        Refused::CONSTRAINTS.len(),
        "the declared list holds a name twice"
    );
    for case in &CASES {
        assert!(
            declared.contains(case.constraint),
            "the case {:?} names {:?}, which this record does not declare",
            case.name,
            case.constraint
        );
    }
}

/// The Done-when of #66, written as its own test rather than left implicit in
/// the case table above.
///
/// One quantity, two source conventions, and the assertion that the two are not
/// comparable until a conversion into one shared convention is recorded for each
/// of them. The subject is one identity because there is no transition record to
/// hang them off: #22 owns that and is open, so "one transition" is held here as
/// one subject and the substitution is stated rather than glossed.
#[test]
fn two_intensities_are_not_comparable_until_both_conversions_are_recorded() {
    let subject = "co-1-0";

    let from_the_millimetre_catalogue = quoted(1.2e-4, millimetre());
    let from_the_infrared_database = quoted(3.4e-21, infrared());

    let refused = compare(&from_the_millimetre_catalogue, &from_the_infrared_database)
        .expect_err("two conventions with nothing recorded between them");
    assert_eq!(
        refused.constraint(),
        "two intensities across conventions with no conversion recorded"
    );

    // One conversion is not enough, and this is the leg that would pass if the
    // operation quietly took the first value's convention as the reference.
    let mut only_the_first_converted = from_the_millimetre_catalogue.clone();
    only_the_first_converted
        .record(RecordedConversion::new(millimetre(), common(), 4.16e-19).expect("a usable factor"))
        .expect("its own convention");
    compare(&only_the_first_converted, &from_the_infrared_database)
        .expect_err("one conversion recorded out of the two the shared convention needs");

    let (first, second) = both_converted_into_the_common_convention();
    let Comparison {
        at,
        first: a,
        second: b,
    } = compare(&first, &second).expect("both conversions recorded");

    assert_eq!(at, common(), "the answer names the convention it is in");
    assert!(a > 0.0 && b > 0.0, "both values arrive, for {subject}");
    assert_eq!(a, 1.2e-4 * 4.16e-19);
    assert_eq!(b, 3.4e-21 * 1.07);
}

/// The refusal says which part differs, rather than that something does.
///
/// This is the difference between a message somebody can act on and one they
/// have to investigate, and it is the reason `differences_from` returns the parts
/// rather than a boolean.
#[test]
fn the_refusal_names_the_part_that_differs() {
    let same_table_different_temperature = Convention::new(
        ReferenceTemperature::kelvin(300.0).expect("300 K is a temperature"),
        Unit::new("cm-1/(molecule cm-2)"),
        PartitionFunctionId::new("hitran-2020-tips").expect("a named tabulation"),
    )
    .expect("all three parts");

    let refused = compare(
        &quoted(3.4e-21, infrared()),
        &quoted(3.4e-21, same_table_different_temperature),
    )
    .expect_err("one part differs, so they are not one convention");

    let said = refused.to_string();
    assert!(
        said.contains("the reference temperature"),
        "the message must name the reference temperature, got {said:?}"
    );
    assert!(
        !said.contains("the partition function"),
        "the message must not name a part that agrees, got {said:?}"
    );

    // And the whole of the difference where every part differs, so the message is
    // not one that names the first thing it finds and stops.
    let all_three = compare(&quoted(1.2e-4, millimetre()), &quoted(3.4e-21, infrared()))
        .expect_err("every part differs")
        .to_string();
    for part in [
        "the reference temperature",
        "the unit",
        "the partition function",
    ] {
        assert!(
            all_three.contains(part),
            "the message must name {part:?}, got {all_three:?}"
        );
    }
}

/// A value already quoted in the convention asked for is returned as it was
/// read, and that is not a conversion.
///
/// Worth a test of its own because the cheap way to write `in_convention` is to
/// look for a recorded conversion first, which would refuse the case where none
/// is needed and push callers towards recording an identity conversion with a
/// factor of one. That factor is a real number pretending to be a no-op.
#[test]
fn a_value_in_the_convention_asked_for_needs_no_conversion() {
    let intensity = quoted(3.4e-21, infrared());
    assert_eq!(
        intensity
            .in_convention(&infrared())
            .expect("its own convention"),
        3.4e-21
    );
    assert_eq!(intensity.reachable_conventions().len(), 1);
}

/// What this file does not cover, printed by every run.
#[test]
fn what_this_file_does_not_cover_is_named() {
    let not_covered = [
        (
            "an intensity constructed with a part of its convention missing",
            "unrepresentable rather than refused: the fields are private and the one \
             constructor takes all three, so there is no value for a case to build and \
             a test cannot demonstrate the absence of a route",
        ),
        (
            "whether a recorded conversion factor is the right factor",
            "a claim about physics rather than about the record; the tabulation is named \
             so a reader can check it, and nothing here evaluates a partition function",
        ),
        (
            "the direction between an intensity and an Einstein coefficient",
            "there is no transition record to hold an Einstein coefficient, which is #22",
        ),
        (
            "a molecular level identity and its quantum numbers",
            "decided in docs/decisions/molecules.md and built with the level record, \
             which is #21 and #22",
        ),
    ];

    println!("intensity conventions does not cover:");
    for (constraint, why) in not_covered {
        println!("  {constraint}: {why}");
    }

    // The list is prose and nothing reads it. What is refusable is that every
    // constraint this record does declare has a case, which is the test above.
    assert_eq!(not_covered.len(), 4);
}

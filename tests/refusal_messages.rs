//! Every refusal in this crate says what it is about.
//!
//! A refusal reaches a caller as text. The type carries the reason, the
//! `Display` implementation is where that reason becomes a sentence, and a
//! caller who prints it is the only reader most refusals ever get. An
//! implementation that writes nothing satisfies the trait, compiles, and turns
//! every refusal below into a blank line at the point somebody needed to know
//! why the arithmetic declined to answer.
//!
//! Nothing else in this tree asserts any of these messages. That is measurable
//! rather than supposed: replacing each of the six implementations named here
//! with one that writes nothing left the whole suite green, which is what a
//! seeded fault run found and what this file exists to end.
//!
//! The messages are pinned exactly rather than searched for a keyword. A
//! substring assertion passes when two variants of one enum swap their
//! sentences, and swapping them is the mistake available here: the variants
//! differ by a few words and sit next to each other in a match.

use linienbuch::register::rounding::NotANumber;
use linienbuch::register::uncertainty::{NoNumber, Refused as UncertaintyRefused, Uncertainty};
use linienbuch::spectroscopy::accuracy::NotAGrade;
use linienbuch::spectroscopy::propagation::{Precondition, Refused as PropagationRefused};

/// The refusal an absent uncertainty produces when a number was needed.
#[test]
fn no_number_says_which_number_is_missing() {
    assert_eq!(
        NoNumber.to_string(),
        "the source quoted no uncertainty, and there is no number to use"
    );
}

/// The two ways a quoted uncertainty is refused before it exists.
#[test]
fn a_refused_uncertainty_says_which_half_was_wrong() {
    assert_eq!(
        UncertaintyRefused::Negative.to_string(),
        "an uncertainty may not be negative"
    );
    assert_eq!(
        UncertaintyRefused::NotFinite.to_string(),
        "an uncertainty must be a finite number"
    );
}

/// The three ways a grade fails to parse.
///
/// Each one carries the text it refused, where there was any, because a caller
/// reading a file of grades needs to know which entry the message is about.
#[test]
fn a_refused_grade_names_what_was_not_a_grade() {
    assert_eq!(NotAGrade::Empty.to_string(), "empty grade");
    assert_eq!(
        NotAGrade::UnknownLetters("Q".to_owned()).to_string(),
        "\"Q\" is not on the published scale"
    );
    assert_eq!(
        NotAGrade::UnknownSuffix("*".to_owned()).to_string(),
        "\"*\" follows the letters and is not a prime"
    );
}

/// The refusal a value that is not finite produces.
#[test]
fn a_value_with_no_rendering_says_why_it_has_none() {
    assert_eq!(
        NotANumber.to_string(),
        "a value that is not finite has no rendering"
    );
}

/// Each precondition reads as the sentence a caller has to establish.
///
/// Over `Precondition::ALL` rather than over a list written here, so a
/// precondition added without a sentence is refused rather than skipped.
#[test]
fn every_precondition_reads_as_a_statement_about_the_line() {
    let said: Vec<String> = Precondition::ALL
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    assert_eq!(
        said,
        [
            "the species does not contribute to the continuous opacity or the electron pressure",
            "the line is not blended",
            "the level populations are in local thermodynamic equilibrium",
            "no parameter was calibrated by requiring no trend with line strength",
            "the line position is known well enough to neglect",
            "the level energies are known well enough to neglect",
        ]
    );
}

/// The five ways a propagation produces no number.
///
/// The two that carry a precondition quote it inside the sentence, so the
/// reader is told which one rather than that there was one. The last wraps the
/// refusal it came from and has to carry that text through rather than
/// replacing it with its own.
#[test]
fn a_refused_propagation_says_which_condition_stopped_it() {
    assert_eq!(
        PropagationRefused::DoesNotHold(Precondition::NotBlended).to_string(),
        "the caller established that the line is not blended is false, and the mapping this \
         board propagates by does not hold there"
    );
    assert_eq!(
        PropagationRefused::NotEstablished(Precondition::NotBlended).to_string(),
        "nobody established that the line is not blended, so a number here would rest on an \
         assumption nobody made"
    );
    assert_eq!(
        PropagationRefused::NoNumberToCarry.to_string(),
        "the claim quoted no uncertainty, and there is nothing to carry"
    );
    assert_eq!(
        PropagationRefused::NoDraws.to_string(),
        "a sampled propagation with no draws has an empty sample"
    );
    assert_eq!(
        PropagationRefused::NotAnUncertainty(UncertaintyRefused::Negative).to_string(),
        "what came back is not one: an uncertainty may not be negative"
    );
}

/// No two of the messages above are the same sentence.
///
/// The assertions are exact, so a swap between two variants is caught by the
/// pair of them. This is the same property stated once over the whole set, and
/// it is what would catch a seventh refusal added with a copied message.
#[test]
fn no_two_refusals_say_the_same_thing() {
    let mut said: Vec<String> = vec![
        NoNumber.to_string(),
        UncertaintyRefused::Negative.to_string(),
        UncertaintyRefused::NotFinite.to_string(),
        NotAGrade::Empty.to_string(),
        NotANumber.to_string(),
        PropagationRefused::NoNumberToCarry.to_string(),
        PropagationRefused::NoDraws.to_string(),
    ];
    said.extend(
        Precondition::ALL
            .iter()
            .map(std::string::ToString::to_string),
    );
    said.extend(
        Precondition::ALL
            .iter()
            .map(|p| PropagationRefused::DoesNotHold(*p).to_string()),
    );

    for one in &said {
        assert!(!one.trim().is_empty(), "a refusal with no message");
    }

    let mut sorted = said.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        said.len(),
        "two refusals carry the same message"
    );
}

/// The refusal an absent uncertainty produces is the one printed above.
///
/// Here rather than in `tests/uncertainty_formatting.rs` because what it pins
/// is the message, and the route to it is the part that would otherwise be
/// assumed: `widest` is the only way to read a number out of this type.
#[test]
fn the_message_is_reached_by_the_route_a_caller_takes() {
    let refusal = Uncertainty::Absent
        .widest()
        .expect_err("an absent uncertainty has no widest half");
    assert_eq!(
        refusal.to_string(),
        "the source quoted no uncertainty, and there is no number to use"
    );
}

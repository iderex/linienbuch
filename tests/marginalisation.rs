//! Every way a set of competing claims fails to become one answer, and the one
//! way it succeeds.
//!
//! Each refusal below has a fixture that violates exactly it and a neighbour
//! one change away that is not refused, which is what `CONTRIBUTING.md` asks of
//! a guard before it ships. The neighbour is the half that costs something: a
//! refusal that fires on everything proves nothing, and several of these are
//! one field apart from a set that comes through.
//!
//! The three preconditions the issue names are here under their own names.
//! `two_claims_about_different_quantities_are_refused` and its neighbours are
//! the first. `two_claims_resting_on_one_measurement_are_refused` is the
//! second, and it is delegated rather than reimplemented, so the test asserts
//! the refusal arrives with the shared end still named. The width policy is the
//! third and has two directions, because the issue says the set either leaves
//! such a claim out explicitly or the operation refuses, and both have to work.
//!
//! The worked case at the end is the shape of the board's headline sentence.
//! Its weighting is a fixture and not a recommendation: entry 7 of #1 puts the
//! table in a record, and nothing in `src/` ships one.

use linienbuch::register::ancestry::{NotIndependent, Terminal};
use linienbuch::register::claims::{
    Ancestor, Claim, Claims, Derivation, Edge, Method, MethodClass, QuantityId, SubjectId, Unit,
};
use linienbuch::register::marginalisation::{
    Marginal, NotAWeighting, Refused, Weighting, WithoutAWidth, marginalise,
};
use linienbuch::register::provenance::{ClaimId, Digest, DigestAlgorithm, SourceId};
use linienbuch::register::uncertainty::{self, Uncertainty};

fn digest() -> Digest {
    Digest::new(
        DigestAlgorithm::Sha256,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    )
    .expect("a well formed digest")
}

fn id(name: &str) -> ClaimId {
    ClaimId::new(name)
}

/// One claim, with everything the tests below vary passed in and everything
/// they do not held fixed.
fn claim(name: &str, method: Method, value: f64, quoted: Uncertainty) -> Claim {
    Claim {
        id: id(name),
        quantity: QuantityId::new("log-gf"),
        about: SubjectId::new("fe-i-4045"),
        value,
        unit: Unit::new("dex"),
        uncertainty: quoted,
        method,
        year: Some(1988),
        source: SourceId::new(name),
        snapshot: digest(),
    }
}

fn measured(name: &str, value: f64, width: f64) -> Claim {
    claim(
        name,
        Method::MeasuredInLaboratory,
        value,
        Uncertainty::symmetric(width).expect("a width"),
    )
}

fn computed(name: &str, value: f64, width: f64) -> Claim {
    claim(
        name,
        Method::Computed {
            code: Some("cowan".to_owned()),
            approximation: None,
        },
        value,
        Uncertainty::symmetric(width).expect("a width"),
    )
}

fn semi_empirical(name: &str, value: f64, width: f64) -> Claim {
    claim(
        name,
        Method::SemiEmpirical,
        value,
        Uncertainty::symmetric(width).expect("a width"),
    )
}

fn register(of: Vec<Claim>) -> Claims {
    let mut held = Claims::new();
    for one in of {
        held.add(one).expect("a claim is accepted");
    }
    held
}

/// A table with one row per category, in the order the categories are declared.
fn table(
    measured: f64,
    computed: f64,
    semi_empirical: f64,
    calibrated: f64,
    compiled: f64,
) -> [(MethodClass, f64); MethodClass::ALL.len()] {
    [
        (MethodClass::MeasuredInLaboratory, measured),
        (MethodClass::Computed, computed),
        (MethodClass::SemiEmpirical, semi_empirical),
        (MethodClass::Calibrated, calibrated),
        (MethodClass::Compiled, compiled),
    ]
}

/// The fixture weighting. Not a recommendation and not a default: it exists so
/// that the tests below have a named table to hand in, and the numbers in it
/// are chosen to be unequal so that a bug that ignores the weights shows.
fn a_weighting() -> Weighting {
    Weighting::named("fixture-by-category", table(4.0, 1.0, 2.0, 1.0, 1.0))
        .expect("a well formed table")
}

fn over(names: &[&str]) -> Vec<ClaimId> {
    names.iter().map(|name| id(name)).collect()
}

// --- The weighting, and the five ways a table is not one ---------------------

#[test]
fn a_table_with_one_row_per_category_is_a_weighting() {
    let weighting = Weighting::named("fixture-by-category", table(4.0, 1.0, 2.0, 1.0, 1.0))
        .expect("a well formed table");
    assert_eq!(weighting.name(), "fixture-by-category");
    assert_eq!(weighting.of(MethodClass::MeasuredInLaboratory), 4.0);
    assert_eq!(weighting.of(MethodClass::Compiled), 1.0);
}

#[test]
fn a_weighting_with_no_name_is_refused() {
    assert_eq!(
        Weighting::named("   ", table(4.0, 1.0, 2.0, 1.0, 1.0)),
        Err(NotAWeighting::Unnamed)
    );
}

#[test]
fn a_table_naming_one_category_twice_is_refused_by_the_category_it_left_out() {
    // One row per category and five rows, so a category written twice is a
    // category missing, and the refusal names the one that has no weight.
    let twice = [
        (MethodClass::MeasuredInLaboratory, 4.0),
        (MethodClass::MeasuredInLaboratory, 1.0),
        (MethodClass::SemiEmpirical, 2.0),
        (MethodClass::Calibrated, 1.0),
        (MethodClass::Compiled, 1.0),
    ];
    assert_eq!(
        Weighting::named("two rows for one category", twice),
        Err(NotAWeighting::NoWeightFor(MethodClass::Computed))
    );
}

#[test]
fn a_weight_that_is_not_a_number_is_refused() {
    assert_eq!(
        Weighting::named("not a number", table(4.0, f64::NAN, 2.0, 1.0, 1.0)),
        Err(NotAWeighting::NotFinite(MethodClass::Computed))
    );
}

#[test]
fn a_negative_weight_is_refused() {
    assert_eq!(
        Weighting::named("below zero", table(4.0, -1.0, 2.0, 1.0, 1.0)),
        Err(NotAWeighting::Negative(MethodClass::Computed))
    );
}

#[test]
fn a_table_that_is_zero_everywhere_is_refused() {
    assert_eq!(
        Weighting::named("nothing", table(0.0, 0.0, 0.0, 0.0, 0.0)),
        Err(NotAWeighting::NothingWeighs)
    );
    // The neighbour, one number away: a table that is zero everywhere but one.
    assert!(Weighting::named("almost nothing", table(0.0, 0.0, 0.0, 0.0, 1.0)).is_ok());
}

// --- The set, and what it has to be before anything is read off it -----------

#[test]
fn no_claims_is_refused_rather_than_answered_with_nothing() {
    let held = register(vec![measured("nist", -0.28, 0.05)]);
    assert_eq!(
        marginalise(&held, &[], &a_weighting(), WithoutAWidth::Refuse),
        Err(Refused::NoClaims)
    );
    // One claim is a set of one and is answered, not refused.
    assert!(
        marginalise(
            &held,
            &over(&["nist"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        )
        .is_ok()
    );
}

#[test]
fn a_claim_this_register_does_not_hold_is_refused_rather_than_skipped() {
    let held = register(vec![measured("nist", -0.28, 0.05)]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist", "kurucz"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::UnknownClaim(id("kurucz")))
    );
}

#[test]
fn two_claims_about_different_quantities_are_refused() {
    let mut other = computed("kurucz", -0.41, 0.12);
    other.quantity = QuantityId::new("einstein-a");
    let held = register(vec![measured("nist", -0.28, 0.05), other]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist", "kurucz"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::NotOneSubject {
            left: id("nist"),
            right: id("kurucz"),
        })
    );
}

#[test]
fn two_claims_about_different_subjects_are_refused() {
    let mut other = computed("kurucz", -0.41, 0.12);
    other.about = SubjectId::new("fe-i-4383");
    let held = register(vec![measured("nist", -0.28, 0.05), other]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist", "kurucz"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::NotOneSubject {
            left: id("nist"),
            right: id("kurucz"),
        })
    );
    // The neighbour, one field back: the same pair about one subject comes
    // through.
    let together = register(vec![
        measured("nist", -0.28, 0.05),
        computed("kurucz", -0.41, 0.12),
    ]);
    assert!(
        marginalise(
            &together,
            &over(&["nist", "kurucz"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        )
        .is_ok()
    );
}

#[test]
fn two_claims_written_in_different_units_are_refused() {
    let mut other = computed("kurucz", -0.41, 0.12);
    other.unit = Unit::new("log10");
    let held = register(vec![measured("nist", -0.28, 0.05), other]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist", "kurucz"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::NotOneUnit {
            left: id("nist"),
            right: id("kurucz"),
        })
    );
}

#[test]
fn a_value_that_is_not_a_number_is_refused_rather_than_averaged() {
    // The record holds the value as a bare number with no constructor in front
    // of it, so this is reachable, and one of these turns every number
    // downstream into the same thing without saying it did.
    let held = register(vec![
        measured("nist", -0.28, 0.05),
        computed("kurucz", f64::NAN, 0.12),
    ]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist", "kurucz"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::NotANumber(id("kurucz")))
    );
}

// --- The second precondition, which is decided elsewhere ---------------------

#[test]
fn two_claims_resting_on_one_measurement_are_refused_with_it_named() {
    let mut held = register(vec![
        claim(
            "one",
            Method::Compiled,
            -0.28,
            Uncertainty::symmetric(0.05).expect("a width"),
        ),
        claim(
            "two",
            Method::Compiled,
            -0.31,
            Uncertainty::symmetric(0.07).expect("a width"),
        ),
        measured("blackwell-1982", -0.29, 0.04),
    ]);
    for quoting in ["one", "two"] {
        held.add_edge(Edge {
            from: id(quoting),
            to: Ancestor::Claim(id("blackwell-1982")),
            derivation: Derivation::Quotation,
        })
        .expect("an edge between two held claims is accepted");
    }

    let refused = marginalise(
        &held,
        &over(&["one", "two"]),
        &a_weighting(),
        WithoutAWidth::Refuse,
    );
    match refused {
        Err(Refused::NotIndependent(NotIndependent::SharedAncestor { ancestor, .. })) => {
            assert_eq!(ancestor, Terminal::Origin(id("blackwell-1982")));
        }
        other => panic!("the shared measurement must be named, got {other:?}"),
    }

    // The neighbour, one edge away: the second claim quotes nobody and is its
    // own origin, so the two rest on different ends and come through.
    let apart = register(vec![
        measured("one", -0.28, 0.05),
        measured("two", -0.31, 0.07),
    ]);
    assert!(
        marginalise(
            &apart,
            &over(&["one", "two"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        )
        .is_ok()
    );
}

// --- The third precondition, in both of its directions -----------------------

#[test]
fn a_claim_quoting_no_width_is_refused_where_the_caller_asked_to_be() {
    let held = register(vec![
        measured("nist", -0.28, 0.05),
        claim("kurucz", Method::SemiEmpirical, -0.41, Uncertainty::Absent),
    ]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist", "kurucz"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::NoWidth(id("kurucz")))
    );
}

#[test]
fn the_same_set_leaves_it_out_and_names_it_where_the_caller_asked_for_that() {
    let held = register(vec![
        measured("nist", -0.28, 0.05),
        claim("kurucz", Method::SemiEmpirical, -0.41, Uncertainty::Absent),
    ]);
    let answer = marginalise(
        &held,
        &over(&["nist", "kurucz"]),
        &a_weighting(),
        WithoutAWidth::LeaveOut,
    )
    .expect("the set comes through with the claim left out");

    assert_eq!(answer.formation().left_out, vec![id("kurucz")]);
    assert_eq!(answer.formation().parts.len(), 1);
    assert_eq!(answer.formation().parts[0].claim, id("nist"));
    // The claim that was dropped is named rather than counted, so the answer
    // cannot be read as one that had nothing to drop.
    assert!(!answer.formation().left_out.is_empty());
}

#[test]
fn a_set_where_every_claim_was_left_out_is_refused() {
    let held = register(vec![
        claim(
            "nist",
            Method::MeasuredInLaboratory,
            -0.28,
            Uncertainty::Absent,
        ),
        claim("kurucz", Method::SemiEmpirical, -0.41, Uncertainty::Absent),
    ]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist", "kurucz"]),
            &a_weighting(),
            WithoutAWidth::LeaveOut
        ),
        Err(Refused::NothingLeft)
    );
}

// --- The weights meeting a set they have nothing to say about ----------------

#[test]
fn a_weighting_that_gives_this_whole_set_zero_is_refused() {
    let held = register(vec![
        computed("kurucz", -0.41, 0.12),
        computed("cowan", -0.44, 0.15),
    ]);
    let nothing_for_computed =
        Weighting::named("nothing for computed", table(4.0, 0.0, 2.0, 1.0, 1.0))
            .expect("a well formed table");
    assert_eq!(
        marginalise(
            &held,
            &over(&["kurucz", "cowan"]),
            &nothing_for_computed,
            WithoutAWidth::Refuse
        ),
        Err(Refused::NoWeightOverThisSet)
    );

    // The neighbour, one row away: the same set under a table that gives that
    // category any weight at all.
    let something = Weighting::named("something for computed", table(4.0, 1.0, 2.0, 1.0, 1.0))
        .expect("a well formed table");
    assert!(
        marginalise(
            &held,
            &over(&["kurucz", "cowan"]),
            &something,
            WithoutAWidth::Refuse
        )
        .is_ok()
    );
}

// --- A width that was not built by its own constructor -----------------------

#[test]
fn a_half_that_is_not_a_number_is_refused() {
    // Reachable because the state has public fields, so a caller can write one
    // that its constructor would have refused.
    let held = register(vec![claim(
        "nist",
        Method::MeasuredInLaboratory,
        -0.28,
        Uncertainty::Quoted {
            minus: f64::NAN,
            plus: 0.05,
        },
    )]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::NotAWidth(uncertainty::Refused::NotFinite))
    );
}

#[test]
fn a_negative_half_is_refused_rather_than_squared_into_a_positive_one() {
    // The near miss worth having: squaring a half is what the arithmetic does,
    // and squaring hides the sign. Refusing before the square is the only place
    // this can be caught.
    let held = register(vec![claim(
        "nist",
        Method::MeasuredInLaboratory,
        -0.28,
        Uncertainty::Quoted {
            minus: -0.05,
            plus: 0.05,
        },
    )]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::NotAWidth(uncertainty::Refused::Negative))
    );
}

#[test]
fn a_half_too_large_to_square_is_refused_rather_than_answered_with_infinity() {
    let held = register(vec![measured("nist", -0.28, 1.0e200)]);
    assert_eq!(
        marginalise(
            &held,
            &over(&["nist"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        ),
        Err(Refused::NotAWidth(uncertainty::Refused::NotFinite))
    );

    // The neighbour, one exponent short of the square overflowing.
    let smaller = register(vec![measured("nist", -0.28, 1.0e150)]);
    assert!(
        marginalise(
            &smaller,
            &over(&["nist"]),
            &a_weighting(),
            WithoutAWidth::Refuse
        )
        .is_ok()
    );
}

// --- What the answer carries -------------------------------------------------

#[test]
fn the_answer_carries_the_weighting_and_every_component_of_the_mixture() {
    let held = register(vec![
        measured("nist", -0.28, 0.05),
        computed("kurucz", -0.41, 0.12),
        semi_empirical("meggers", -0.33, 0.08),
    ]);
    let answer = marginalise(
        &held,
        &over(&["nist", "kurucz", "meggers"]),
        &a_weighting(),
        WithoutAWidth::Refuse,
    )
    .expect("three independent claims about one quantity");

    assert_eq!(answer.formation().weighting, "fixture-by-category");
    assert_eq!(answer.quantity(), &QuantityId::new("log-gf"));
    assert_eq!(answer.about(), &SubjectId::new("fe-i-4045"));
    assert_eq!(answer.unit(), &Unit::new("dex"));

    // Every claim that went in is in the mixture, under its own category, with
    // the share the table gave it. The claims are still there afterwards, which
    // is the property #36 is about and this operation must not break.
    let parts = &answer.formation().parts;
    assert_eq!(parts.len(), 3);
    assert_eq!(held.len(), 3);
    let shares: Vec<f64> = parts.iter().map(|part| part.weight).collect();
    assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1.0e-12);
    assert!((shares[0] - 4.0 / 7.0).abs() < 1.0e-12);
    assert!((shares[1] - 1.0 / 7.0).abs() < 1.0e-12);
    assert!((shares[2] - 2.0 / 7.0).abs() < 1.0e-12);
    assert_eq!(parts[0].class, MethodClass::MeasuredInLaboratory);
    assert_eq!(parts[1].class, MethodClass::Computed);
    assert_eq!(parts[2].class, MethodClass::SemiEmpirical);

    // The mean is the weighted one and not the arithmetic one, which is the
    // number a run that ignored the table would produce.
    let weighted = (4.0 * -0.28 + 1.0 * -0.41 + 2.0 * -0.33) / 7.0;
    let unweighted = (-0.28 + -0.41 + -0.33) / 3.0;
    assert!((answer.value() - weighted).abs() < 1.0e-12);
    assert!((answer.value() - unweighted).abs() > 1.0e-4);
}

#[test]
fn the_mixture_is_never_narrower_than_the_narrowest_claim_in_it() {
    // Per half, which is the bound the arithmetic actually holds. The weights
    // are a share of one, so the weighted mean of the squares is at least the
    // smallest of them, and the spread term only adds.
    for widths in [[0.05, 0.12, 0.08], [0.4, 0.4, 0.4], [0.01, 0.9, 0.5]] {
        let held = register(vec![
            measured("nist", -0.28, widths[0]),
            computed("kurucz", -0.41, widths[1]),
            semi_empirical("meggers", -0.33, widths[2]),
        ]);
        let answer = marginalise(
            &held,
            &over(&["nist", "kurucz", "meggers"]),
            &a_weighting(),
            WithoutAWidth::Refuse,
        )
        .expect("three independent claims about one quantity");

        let narrowest = widths.iter().copied().fold(f64::INFINITY, f64::min);
        let (lower, upper) = both_halves(&answer);
        assert!(
            lower >= narrowest && upper >= narrowest,
            "a mixture of {widths:?} came back narrower than {narrowest}"
        );
    }
}

#[test]
fn claims_that_agree_exactly_come_back_at_their_own_width() {
    // The case that must not be advertised as widening. Three sources at one
    // value with one width have no spread between them, so the honest answer is
    // that width and not a larger one.
    let held = register(vec![
        measured("nist", -0.28, 0.05),
        computed("kurucz", -0.28, 0.05),
        semi_empirical("meggers", -0.28, 0.05),
    ]);
    let answer = marginalise(
        &held,
        &over(&["nist", "kurucz", "meggers"]),
        &a_weighting(),
        WithoutAWidth::Refuse,
    )
    .expect("three independent claims about one quantity");

    assert!(answer.formation().between.abs() < 1.0e-12);
    let (lower, upper) = both_halves(&answer);
    assert!((lower - 0.05).abs() < 1.0e-12);
    assert!((upper - 0.05).abs() < 1.0e-12);
}

/// The two halves of an answer's width, as numbers.
fn both_halves(answer: &Marginal) -> (f64, f64) {
    match answer.spread() {
        Uncertainty::Quoted { minus, plus } => (minus, plus),
        Uncertainty::Absent => panic!("an answer that came through quotes a width"),
    }
}

// --- The worked case ---------------------------------------------------------

/// The shape of the board's headline sentence, in one command.
///
/// Three sources compete over one line. The value a reader would otherwise have
/// quoted is the one from the source the table weights highest, and it carries
/// that source's own width. Marginalising over the three carries the spread
/// between them as well, and comes out wider.
///
/// Run with the output shown:
///
///     cargo test --locked --test marginalisation -- --nocapture the_worked_case
#[test]
fn the_worked_case() {
    let held = register(vec![
        measured("nist", -0.28, 0.05),
        computed("kurucz", -0.41, 0.12),
        semi_empirical("meggers", -0.33, 0.08),
    ]);
    let answer = marginalise(
        &held,
        &over(&["nist", "kurucz", "meggers"]),
        &a_weighting(),
        WithoutAWidth::Refuse,
    )
    .expect("three independent claims about one quantity");

    // What one source says on its own, which is what a reader taking the
    // preferred compilation and stopping there would quote.
    let one_source = 0.05;
    let (lower, upper) = both_halves(&answer);

    println!("weighting: {}", answer.formation().weighting);
    for part in &answer.formation().parts {
        println!(
            "  {} is {} at {} with a share of {:.4}",
            part.claim, part.class, part.value, part.weight
        );
    }
    println!("one source alone: {} +/- {one_source}", -0.28);
    println!(
        "marginalised:     {:.6} -{lower:.6} +{upper:.6}",
        answer.value()
    );
    println!(
        "  of which between the sources: {:.6}",
        answer.formation().between
    );

    assert!(lower > one_source);
    assert!(upper > one_source);
    // And the claims are all still in the register afterwards.
    assert_eq!(held.len(), 3);
}

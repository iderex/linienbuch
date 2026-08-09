//! What the propagation refuses, and the independent calculation it agrees
//! with.
//!
//! `docs/decisions/propagation.md` is the record this suite is written against.
//! The two things worth stating before the cases.
//!
//! Every refusal fixture here answers five of the six preconditions and differs
//! from a fixture that propagates by one answer. A fixture that leaves all six
//! unanswered would be refused by whichever one is looked at first, and would
//! not show which refusal it tripped.
//!
//! The comparison against the sampled route is the only check on the exchange
//! of the halves that does not share arithmetic with the thing it checks. The
//! near miss it is written for is the symmetric case: an implementation that
//! carries the halves across unswapped produces the same bytes as a correct one
//! for every symmetric uncertainty, so the fixture that decides anything is the
//! asymmetric one.

use linienbuch::register::uncertainty::{self, Uncertainty};
use linienbuch::spectroscopy::propagation::{
    Answer, Conditions, DERIVATIVE, Precondition, Refused, analytic, monte_carlo,
};

/// The draw count and the seed for every sampled run here.
///
/// Fixed so that two runs of one commit produce the same numbers. The count is
/// chosen against the tolerance below rather than picked: the root mean square
/// of a side holding n draws has a relative standard error of one over the
/// square root of twice n, so the smaller side of the worked case below carries
/// about three parts in a thousand and the tolerance is ten times that.
const DRAWS: usize = 200_000;
const SEED: u64 = 20_260_809;

/// How far the sampled answer may sit from the analytic one, as a fraction.
const TOLERANCE: f64 = 0.03;

/// Everything answered, which is the only state that propagates.
fn everything_established() -> Conditions {
    Precondition::ALL
        .into_iter()
        .fold(Conditions::nothing_established(), |conditions, one| {
            conditions.with(one, Answer::Holds)
        })
}

fn quoted(minus: f64, plus: f64) -> Uncertainty {
    Uncertainty::asymmetric(minus, plus).expect("the fixture is a valid uncertainty")
}

fn halves_of(carried: Uncertainty) -> (f64, f64) {
    match carried {
        Uncertainty::Quoted { minus, plus } => (minus, plus),
        Uncertainty::Absent => panic!("a propagation may not produce an absent uncertainty"),
    }
}

/// The clause #37 names on its own: an absent uncertainty cannot enter.
///
/// Both routes, because a refusal held on one of them is a refusal a caller
/// reaches around by asking for the other.
#[test]
fn a_value_whose_uncertainty_is_absent_cannot_enter_the_propagation() {
    let established = everything_established();

    assert_eq!(
        analytic(Uncertainty::Absent, &established),
        Err(Refused::NoNumberToCarry)
    );
    assert_eq!(
        monte_carlo(Uncertainty::Absent, &established, DRAWS, SEED),
        Err(Refused::NoNumberToCarry)
    );

    // The neighbour, one state away: the same claim with a quoted uncertainty
    // of zero, which is a source saying the value is exact rather than a source
    // saying nothing. It propagates.
    assert_eq!(
        analytic(quoted(0.0, 0.0), &established),
        Ok(quoted(0.0, 0.0))
    );
}

/// A precondition nobody established refuses, and names itself.
///
/// One case per precondition, each differing from a propagating fixture by one
/// answer, so the refusal reported is the one the fixture is about.
#[test]
fn a_precondition_nobody_established_refuses_and_says_which() {
    for one in Precondition::ALL {
        let established = everything_established().with(one, Answer::NotEstablished);
        assert_eq!(
            analytic(quoted(0.05, 0.12), &established),
            Err(Refused::NotEstablished(one)),
            "leaving {one} unanswered must refuse for that reason and no other"
        );
    }

    // The neighbour: the same six, all answered. Nothing is refused.
    assert!(analytic(quoted(0.05, 0.12), &everything_established()).is_ok());
}

/// A precondition established as false refuses, and it is a different refusal.
///
/// The two states are one character apart in a caller's code and lead to
/// different repairs. Collapsing them would tell somebody who established a
/// blend that they had failed to answer.
#[test]
fn a_precondition_established_as_false_is_a_different_refusal() {
    for one in Precondition::ALL {
        let established = everything_established().with(one, Answer::DoesNotHold);
        assert_eq!(
            analytic(quoted(0.05, 0.12), &established),
            Err(Refused::DoesNotHold(one))
        );
        assert_ne!(
            analytic(quoted(0.05, 0.12), &established),
            Err(Refused::NotEstablished(one))
        );
    }
}

/// Nothing established is the default, so a caller who built no conditions at
/// all is refused rather than served the common branch.
#[test]
fn the_default_state_refuses() {
    let nothing = Conditions::nothing_established();
    for one in Precondition::ALL {
        assert_eq!(nothing.answer(one), Answer::NotEstablished);
    }
    assert_eq!(
        analytic(quoted(0.05, 0.12), &nothing),
        Err(Refused::NotEstablished(Precondition::ALL[0]))
    );
}

/// The sampled route refuses an empty sample rather than reporting it as zero.
#[test]
fn a_sampled_propagation_with_no_draws_refuses() {
    let established = everything_established();
    assert_eq!(
        monte_carlo(quoted(0.05, 0.12), &established, 0, SEED),
        Err(Refused::NoDraws)
    );

    // The neighbour, one draw away: a single draw is a bad estimate and is not
    // an empty sample, so it is not refused.
    assert!(monte_carlo(quoted(0.05, 0.12), &established, 1, SEED).is_ok());
}

/// The state is constructible without its constructor, so what comes back is
/// checked rather than assumed.
#[test]
fn halves_that_are_not_an_uncertainty_are_refused() {
    let built_by_hand = Uncertainty::Quoted {
        minus: 0.05,
        plus: -0.12,
    };
    assert_eq!(
        analytic(built_by_hand, &everything_established()),
        Err(Refused::NotAnUncertainty(uncertainty::Refused::Negative))
    );

    let not_a_number = Uncertainty::Quoted {
        minus: 0.05,
        plus: f64::NAN,
    };
    assert_eq!(
        analytic(not_a_number, &everything_established()),
        Err(Refused::NotAnUncertainty(uncertainty::Refused::NotFinite))
    );
}

/// The halves exchange, because the slope is negative.
///
/// This is the case the whole comparison below exists for. An implementation
/// that carries them across in place passes every symmetric fixture.
#[test]
fn an_asymmetric_uncertainty_comes_out_with_its_halves_exchanged() {
    // The exchange follows from the sign, so the sign is asserted where a
    // reader meets the exchange rather than left in the record.
    const { assert!(DERIVATIVE < 0.0) };

    let carried =
        analytic(quoted(0.05, 0.12), &everything_established()).expect("everything is established");
    assert_eq!(halves_of(carried), (0.12, 0.05));

    // The near miss, one number away: symmetric, where an implementation that
    // forgot the exchange is indistinguishable from this one.
    let symmetric =
        analytic(quoted(0.07, 0.07), &everything_established()).expect("everything is established");
    assert_eq!(halves_of(symmetric), (0.07, 0.07));
}

/// The worked case, computed both ways.
///
/// Run with `-- --nocapture` to see the two answers side by side; the record in
/// `docs/decisions/propagation.md` quotes that output and the command that
/// produced it.
#[test]
fn the_two_methods_agree_on_the_worked_case() {
    let established = everything_established();
    let quoted_on_log_gf = quoted(0.05, 0.12);

    let by_hand = analytic(quoted_on_log_gf, &established).expect("everything is established");
    let by_sampling = monte_carlo(quoted_on_log_gf, &established, DRAWS, SEED)
        .expect("everything is established");

    let (hand_lower, hand_upper) = halves_of(by_hand);
    let (sampled_lower, sampled_upper) = halves_of(by_sampling);
    let (input_lower, input_upper) = halves_of(quoted_on_log_gf);

    println!("quoted on log gf    lower {input_lower:.6} upper {input_upper:.6}");
    println!("analytic            lower {hand_lower:.6} upper {hand_upper:.6}");
    println!("sampled             lower {sampled_lower:.6} upper {sampled_upper:.6}");
    println!("sampling            {DRAWS} draws, seed {SEED}");

    for (sampled, exact, side) in [
        (sampled_lower, hand_lower, "lower"),
        (sampled_upper, hand_upper, "upper"),
    ] {
        let apart = (sampled - exact).abs() / exact;
        assert!(
            apart <= TOLERANCE,
            "the {side} half is {apart} of the analytic answer away from it, above {TOLERANCE}"
        );
    }
}

/// The sampled route is a check only while it is reproducible.
#[test]
fn the_sampled_route_is_deterministic_in_its_seed() {
    let established = everything_established();
    let once = monte_carlo(quoted(0.05, 0.12), &established, DRAWS, SEED);
    let again = monte_carlo(quoted(0.05, 0.12), &established, DRAWS, SEED);
    assert_eq!(once, again);

    // The neighbour, one seed away: a different stream, so a different sample.
    let elsewhere = monte_carlo(quoted(0.05, 0.12), &established, DRAWS, SEED + 1);
    assert_ne!(once, elsewhere);
}

/// The worked case's sampled answer, pasted from a run rather than compared to
/// the analytic one.
///
/// The comparison above reads the two conditional root mean squares and nothing
/// else, and neither of them depends on how the draws were split between the
/// two sides: only the sampling error of each does. So the split can be wrong by
/// a factor of thirty and the comparison still passes inside its tolerance.
/// Measured rather than supposed, by seeding that fault for #41: turning the
/// share into a product moved the answer to 0.120199 and 0.049480, which is
/// inside the three per cent the case above allows.
///
/// Six decimals rather than every digit, and that is a deliberate bound. Which
/// side a draw lands on is decided by integer arithmetic and an exact division,
/// so it is the same on every platform; the magnitudes go through `ln` and
/// `cos`, which are not required to be correctly rounded and may differ in the
/// last place between one library and another. Six decimals is far above that
/// and far below the change any wrong split produces.
#[test]
fn the_worked_case_samples_the_numbers_the_record_quotes() {
    let established = everything_established();
    let (lower, upper) = halves_of(
        monte_carlo(quoted(0.05, 0.12), &established, DRAWS, SEED)
            .expect("everything is established"),
    );

    assert_eq!(
        [format!("{lower:.6}"), format!("{upper:.6}")],
        ["0.120107".to_owned(), "0.050158".to_owned()]
    );
}

/// A one-sided uncertainty is the case where the exchange is visible without
/// arithmetic, and it is where an empty side of the sample is legitimate.
#[test]
fn a_one_sided_uncertainty_exchanges_its_empty_side() {
    let established = everything_established();
    let carried = analytic(quoted(0.0, 0.2), &established).expect("everything is established");
    assert_eq!(halves_of(carried), (0.2, 0.0));

    let sampled = monte_carlo(quoted(0.0, 0.2), &established, DRAWS, SEED)
        .expect("everything is established");
    let (sampled_lower, sampled_upper) = halves_of(sampled);
    assert_eq!(sampled_upper, 0.0, "a side with no width draws nothing");
    assert!((sampled_lower - 0.2).abs() / 0.2 <= TOLERANCE);
}

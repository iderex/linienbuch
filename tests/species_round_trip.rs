//! Every upstream spelling the board has seen, into the canonical identity and
//! back into that upstream's spelling.
//!
//! The round trip is the test that catches the failure this record is about. A
//! parser that reads a spectrum number as a charge produces a species that is
//! wrong by one ionisation stage and is otherwise perfectly well formed, so
//! nothing downstream can notice. Rendering back and comparing to the string the
//! source wrote is what notices.

use linienbuch::spectroscopy::species::{Convention, Element, Species, Unparseable};

fn atom(symbol: &str, charge: u8) -> Species {
    let element = Element::from_symbol(symbol).expect("symbol is in the element table");
    Species::atom(element, charge).expect("charge is within the element")
}

/// One row per spelling: the convention, the string a source writes, and the
/// identity it means.
///
/// The charges are the load bearing column. `Fe II`, `FeII` and `Fe_2` are all
/// charge one because those three conventions count spectra, while `Fe+`,
/// `Fe 1+` and `26.01` are charge one because those three count charges. A table
/// where every row for one ion agrees on the charge while disagreeing on the
/// digit is the whole point of the record.
fn spellings() -> Vec<(Convention, &'static str, Species)> {
    vec![
        (Convention::SpectroscopicSpaced, "Fe I", atom("Fe", 0)),
        (Convention::SpectroscopicSpaced, "Fe II", atom("Fe", 1)),
        (Convention::SpectroscopicSpaced, "Ca XX", atom("Ca", 19)),
        (Convention::SpectroscopicSpaced, "H I", atom("H", 0)),
        (Convention::SpectroscopicCompact, "FeI", atom("Fe", 0)),
        (Convention::SpectroscopicCompact, "FeII", atom("Fe", 1)),
        (Convention::SpectroscopicCompact, "NI", atom("N", 0)),
        (Convention::SpectroscopicCompact, "NiI", atom("Ni", 0)),
        (Convention::SpectroscopicCompact, "SiIV", atom("Si", 3)),
        (Convention::SpectroscopicUnderscore, "Fe_1", atom("Fe", 0)),
        (Convention::SpectroscopicUnderscore, "Fe_2", atom("Fe", 1)),
        (Convention::SpectroscopicUnderscore, "Mg_12", atom("Mg", 11)),
        (Convention::ChargeSign, "Fe", atom("Fe", 0)),
        (Convention::ChargeSign, "Fe+", atom("Fe", 1)),
        (Convention::ChargeSign, "Fe2+", atom("Fe", 2)),
        (Convention::ChargeSign, "O6+", atom("O", 6)),
        (Convention::ChargeSpacedSign, "Fe 0+", atom("Fe", 0)),
        (Convention::ChargeSpacedSign, "Fe 1+", atom("Fe", 1)),
        (Convention::ChargeSpacedSign, "Ca 19+", atom("Ca", 19)),
        (Convention::NumericCode, "26.00", atom("Fe", 0)),
        (Convention::NumericCode, "26.01", atom("Fe", 1)),
        (Convention::NumericCode, "01.00", atom("H", 0)),
        (Convention::NumericCode, "92.05", atom("U", 5)),
    ]
}

#[test]
fn every_spelling_parses_to_the_identity_it_means() {
    for (convention, text, expected) in spellings() {
        let parsed = convention
            .parse(text)
            .unwrap_or_else(|e| panic!("{convention:?} refused {text:?}: {e}"));
        assert_eq!(parsed, expected, "{convention:?} parsed {text:?} wrongly");
    }
}

#[test]
fn every_spelling_renders_back_to_itself() {
    for (convention, text, _) in spellings() {
        let parsed = convention
            .parse(text)
            .unwrap_or_else(|e| panic!("{convention:?} refused {text:?}: {e}"));
        assert_eq!(
            convention.render(parsed),
            text,
            "{convention:?} did not render {text:?} back to itself"
        );
    }
}

/// The two families disagree about the digit and agree about the ion, which is
/// the confusion the canonical identity exists to remove. Written as its own
/// test rather than left implicit in the table above, because it is the claim
/// the record makes.
#[test]
fn spectrum_counting_and_charge_counting_conventions_meet_at_one_identity() {
    let by_spectrum = [
        Convention::SpectroscopicSpaced.parse("Fe II"),
        Convention::SpectroscopicCompact.parse("FeII"),
        Convention::SpectroscopicUnderscore.parse("Fe_2"),
    ];
    let by_charge = [
        Convention::ChargeSign.parse("Fe+"),
        Convention::ChargeSpacedSign.parse("Fe 1+"),
        Convention::NumericCode.parse("26.01"),
    ];

    let expected = atom("Fe", 1);
    for parsed in by_spectrum.into_iter().chain(by_charge) {
        assert_eq!(parsed.expect("spelling parses"), expected);
    }
}

/// The off by one, as a fixture. `26.02` counts charges and `Fe II` counts
/// spectra, so reading one as the other lands on the wrong ionisation stage and
/// on a number that looks entirely reasonable.
#[test]
fn a_numeric_code_is_not_a_spectrum_number() {
    let by_code = Convention::NumericCode.parse("26.02").expect("parses");
    let by_spectrum = Convention::SpectroscopicSpaced
        .parse("Fe II")
        .expect("parses");

    assert_ne!(by_code, by_spectrum);
    assert_eq!(by_code, atom("Fe", 2));
    assert_eq!(by_spectrum, atom("Fe", 1));
}

/// Every spelling that must be refused, with the reason it is refused for.
///
/// Each is one change away from something that parses, which is the near miss
/// worth spending the effort on. A file of obvious nonsense would prove that the
/// parser refuses obvious nonsense.
#[test]
fn an_unparseable_species_is_refused_with_its_reason() {
    let refused: Vec<(Convention, &str, Unparseable)> = vec![
        (
            Convention::SpectroscopicSpaced,
            "Xy II",
            Unparseable::UnknownElement("Xy".to_owned()),
        ),
        (
            Convention::SpectroscopicSpaced,
            "Fe IIII",
            Unparseable::MalformedStage("IIII".to_owned()),
        ),
        (
            Convention::SpectroscopicSpaced,
            "Fe",
            Unparseable::MissingStage,
        ),
        (
            Convention::SpectroscopicSpaced,
            "Fe II III",
            Unparseable::MalformedStage("II III".to_owned()),
        ),
        (
            Convention::SpectroscopicCompact,
            "Fe",
            Unparseable::MissingStage,
        ),
        (
            Convention::SpectroscopicCompact,
            "H2O",
            Unparseable::MoreThanOneElement("H2O".to_owned()),
        ),
        (
            Convention::SpectroscopicUnderscore,
            "Fe_0",
            Unparseable::MalformedStage("0".to_owned()),
        ),
        (
            Convention::ChargeSign,
            "Fe1+",
            Unparseable::MalformedStage("1+".to_owned()),
        ),
        (
            Convention::ChargeSpacedSign,
            "Fe 1",
            Unparseable::MalformedStage("1".to_owned()),
        ),
        (
            Convention::NumericCode,
            "93.00",
            Unparseable::UnknownElement("93".to_owned()),
        ),
        (Convention::NumericCode, "26", Unparseable::MissingStage),
        (Convention::ChargeSign, "", Unparseable::Empty),
        (
            Convention::SpectroscopicSpaced,
            "H III",
            Unparseable::MoreChargeThanElectrons {
                symbol: "H",
                charge: 2,
            },
        ),
    ];

    for (convention, text, expected) in refused {
        match convention.parse(text) {
            Ok(species) => panic!("{convention:?} accepted {text:?} as {species:?}"),
            Err(reason) => assert_eq!(reason, expected, "{convention:?} refused {text:?} wrongly"),
        }
    }
}

/// The neighbours of the refusals above. Each is one change from a refused
/// spelling and each must parse, because a parser that refuses these too has
/// proved only that it refuses.
#[test]
fn the_neighbours_of_the_refusals_are_not_refused() {
    let accepted = [
        (Convention::SpectroscopicSpaced, "Xe II"),
        (Convention::SpectroscopicSpaced, "Fe III"),
        (Convention::SpectroscopicCompact, "FeI"),
        (Convention::SpectroscopicCompact, "HI"),
        (Convention::SpectroscopicUnderscore, "Fe_1"),
        (Convention::ChargeSign, "Fe2+"),
        (Convention::ChargeSpacedSign, "Fe 1+"),
        (Convention::NumericCode, "92.00"),
        (Convention::SpectroscopicSpaced, "H II"),
    ];

    for (convention, text) in accepted {
        convention
            .parse(text)
            .unwrap_or_else(|e| panic!("{convention:?} refused the neighbour {text:?}: {e}"));
    }
}

/// A species is a structured identity, so two spellings of one ion are equal and
/// usable as a key. A text species would make these two different rows in every
/// later aggregate, which is the failure the record exists to prevent.
#[test]
fn one_ion_is_one_key_however_it_was_spelled() {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    seen.insert(
        Convention::SpectroscopicSpaced
            .parse("Fe II")
            .expect("parses"),
    );
    seen.insert(Convention::ChargeSign.parse("Fe+").expect("parses"));
    seen.insert(Convention::NumericCode.parse("26.01").expect("parses"));

    assert_eq!(seen.len(), 1);
}

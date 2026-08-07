//! Species and ionisation stage, with one canonical spelling.
//!
//! The same ion is written `Fe II`, `FeII`, `Fe+`, `Fe 1+`, `Fe_2` and `26.01`,
//! and the differences are not only cosmetic. Two of those conventions count
//! spectra and the others count charges, so `Fe II` and `Fe+` are the same ion
//! while `Fe II` and a naive reading of `26.02` are not. An off by one here is
//! not detectable from the value, because a transition probability for the wrong
//! ionisation stage is a perfectly plausible number.
//!
//! So there is one canonical internal identity, one parser per upstream
//! convention, and no free text anywhere. A species that does not parse is
//! refused rather than carried as a string, because a string that nearly matches
//! is what silently splits one species into two rows in every later aggregate.

use std::fmt;

/// A chemical element, held as its atomic number.
///
/// The table stops at uranium. Nothing above it appears in the spectra this
/// board is about, and a table carrying entries nobody has ever needed is a
/// table whose errors nobody ever finds. Extending it is adding rows to
/// `SYMBOLS` and is what a source spelling a heavier element would force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Element(u8);

/// Symbols indexed by atomic number minus one, hydrogen through uranium.
const SYMBOLS: [&str; 92] = [
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U",
];

impl Element {
    /// The element with this atomic number, if the table carries it.
    pub fn from_atomic_number(z: u8) -> Option<Self> {
        if z >= 1 && usize::from(z) <= SYMBOLS.len() {
            Some(Element(z))
        } else {
            None
        }
    }

    /// The element with this symbol, matched case sensitively. `CO` is not
    /// cobalt and `Co` is not carbon monoxide, and a case insensitive match here
    /// would collapse the two.
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        SYMBOLS
            .iter()
            .position(|s| *s == symbol)
            .map(|index| Element(u8::try_from(index + 1).expect("table is shorter than 255")))
    }

    pub fn atomic_number(self) -> u8 {
        self.0
    }

    pub fn symbol(self) -> &'static str {
        SYMBOLS[usize::from(self.0) - 1]
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

/// The canonical identity of a species.
///
/// An enumeration with one variant today rather than a struct, because the
/// molecular case is a different identity and not a longer version of this one.
/// A molecular species is identified by a formula and an isotopologue, and an
/// isotopologue is part of the identity rather than an attribute, since a value
/// for one is not a value for another. That variant lands with #66. Making it a
/// variant means every match in the tree is forced to be revisited when it
/// arrives, which is the whole reason for choosing a language with exhaustive
/// matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Species {
    /// An atom or an atomic ion. `charge` is the number of electrons removed,
    /// so a neutral atom is zero. It is never a spectrum number: the conversion
    /// from a spectrum number happens in the parser for the convention that uses
    /// one, in one place, rather than at each call site.
    Atom { element: Element, charge: u8 },
}

impl Species {
    /// An atom or atomic ion, refusing a charge the element cannot carry.
    pub fn atom(element: Element, charge: u8) -> Result<Self, Unparseable> {
        if charge > element.atomic_number() {
            return Err(Unparseable::MoreChargeThanElectrons {
                symbol: element.symbol(),
                charge,
            });
        }
        Ok(Species::Atom { element, charge })
    }
}

/// Why a spelling was refused.
///
/// A reason rather than a boolean, because "this is not a species" and "this is
/// a species this parser does not cover" are different answers and only one of
/// them is a defect in the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unparseable {
    /// Nothing to parse.
    Empty,
    /// The leading symbol is not in the element table.
    UnknownElement(String),
    /// The stage part is present but is not what the convention spells.
    MalformedStage(String),
    /// The convention wants a stage and none is there.
    MissingStage,
    /// More than one element symbol, so this is a molecule rather than an atom.
    /// Refused by name rather than as an unknown element, because the input is
    /// a species this parser does not cover rather than nonsense. The molecular
    /// identity is #66.
    MoreThanOneElement(String),
    /// The charge exceeds the number of electrons the element has.
    MoreChargeThanElectrons { symbol: &'static str, charge: u8 },
}

impl fmt::Display for Unparseable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unparseable::Empty => write!(f, "empty species"),
            Unparseable::UnknownElement(s) => write!(f, "unknown element symbol {s:?}"),
            Unparseable::MalformedStage(s) => write!(f, "malformed ionisation stage {s:?}"),
            Unparseable::MissingStage => {
                write!(f, "this convention spells a stage and none is present")
            }
            Unparseable::MoreThanOneElement(s) => {
                write!(f, "{s:?} names more than one element, so it is a molecule")
            }
            Unparseable::MoreChargeThanElectrons { symbol, charge } => {
                write!(f, "{symbol} cannot carry a charge of {charge}")
            }
        }
    }
}

/// An upstream spelling convention.
///
/// One parser and one renderer per convention, so that the place a spectrum
/// number becomes a charge is a single line that can be looked at, rather than
/// an assumption spread over every ingest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Convention {
    /// `Fe II`. The numeral is the spectrum, so the charge is one less.
    SpectroscopicSpaced,
    /// `FeII`. The same numeral with the space removed.
    SpectroscopicCompact,
    /// `Fe_2`. The digit is the spectrum, so the charge is one less.
    SpectroscopicUnderscore,
    /// `Fe+`, `Fe2+`, and `Fe` for the neutral. The digit is the charge.
    ChargeSign,
    /// `Fe 1+`. The digit is the charge and is always written.
    ChargeSpacedSign,
    /// `26.01`. Atomic number before the point, charge after it.
    NumericCode,
}

impl Convention {
    /// Read one upstream spelling into the canonical identity.
    pub fn parse(self, text: &str) -> Result<Species, Unparseable> {
        let text = text.trim();
        if text.is_empty() {
            return Err(Unparseable::Empty);
        }
        match self {
            Convention::SpectroscopicSpaced => {
                let (symbol, numeral) = split_once_exactly(text, ' ')?;
                let spectrum = roman_to_number(numeral)
                    .ok_or_else(|| Unparseable::MalformedStage(numeral.to_owned()))?;
                Species::atom(element(symbol)?, spectrum - 1)
            }
            Convention::SpectroscopicCompact => {
                let (symbol, numeral) = split_leading_symbol(text)?;
                if numeral.is_empty() {
                    return Err(Unparseable::MissingStage);
                }
                let spectrum = roman_to_number(numeral)
                    .ok_or_else(|| Unparseable::MalformedStage(numeral.to_owned()))?;
                Species::atom(symbol, spectrum - 1)
            }
            Convention::SpectroscopicUnderscore => {
                let (symbol, digits) = split_once_exactly(text, '_')?;
                let spectrum = digits
                    .parse::<u8>()
                    .ok()
                    .filter(|n| *n >= 1)
                    .ok_or_else(|| Unparseable::MalformedStage(digits.to_owned()))?;
                Species::atom(element(symbol)?, spectrum - 1)
            }
            Convention::ChargeSign => {
                let (symbol, rest) = split_leading_symbol(text)?;
                let charge = match rest {
                    "" => 0,
                    "+" => 1,
                    other => other
                        .strip_suffix('+')
                        .and_then(|digits| digits.parse::<u8>().ok())
                        .filter(|n| *n > 1)
                        .ok_or_else(|| Unparseable::MalformedStage(other.to_owned()))?,
                };
                Species::atom(symbol, charge)
            }
            Convention::ChargeSpacedSign => {
                let (symbol, rest) = split_once_exactly(text, ' ')?;
                let charge = rest
                    .strip_suffix('+')
                    .and_then(|digits| digits.parse::<u8>().ok())
                    .ok_or_else(|| Unparseable::MalformedStage(rest.to_owned()))?;
                Species::atom(element(symbol)?, charge)
            }
            Convention::NumericCode => {
                let (left, right) = split_once_exactly(text, '.')?;
                let z = left
                    .parse::<u8>()
                    .ok()
                    .ok_or_else(|| Unparseable::UnknownElement(left.to_owned()))?;
                let element = Element::from_atomic_number(z)
                    .ok_or_else(|| Unparseable::UnknownElement(left.to_owned()))?;
                let charge = right
                    .parse::<u8>()
                    .ok()
                    .ok_or_else(|| Unparseable::MalformedStage(right.to_owned()))?;
                Species::atom(element, charge)
            }
        }
    }

    /// Write the canonical identity back in this convention's spelling.
    pub fn render(self, species: Species) -> String {
        let Species::Atom { element, charge } = species;
        match self {
            Convention::SpectroscopicSpaced => {
                format!("{element} {}", number_to_roman(charge + 1))
            }
            Convention::SpectroscopicCompact => {
                format!("{element}{}", number_to_roman(charge + 1))
            }
            Convention::SpectroscopicUnderscore => format!("{element}_{}", charge + 1),
            Convention::ChargeSign => match charge {
                0 => element.to_string(),
                1 => format!("{element}+"),
                n => format!("{element}{n}+"),
            },
            Convention::ChargeSpacedSign => format!("{element} {charge}+"),
            Convention::NumericCode => {
                format!("{:02}.{:02}", element.atomic_number(), charge)
            }
        }
    }
}

fn element(symbol: &str) -> Result<Element, Unparseable> {
    Element::from_symbol(symbol).ok_or_else(|| {
        if looks_molecular(symbol) {
            Unparseable::MoreThanOneElement(symbol.to_owned())
        } else {
            Unparseable::UnknownElement(symbol.to_owned())
        }
    })
}

/// Whether what is left after the leading element symbol names a second element.
///
/// The test is an uppercase letter that is not one of the roman numeral
/// characters, because every convention here spells a stage with digits, a plus,
/// or `I`, `V`, `X`, `L` and `C`. So `FeII` is iron and a numeral while `H2O` and
/// `SiO` name a second element. The overlap is real and is why the test is
/// written this way round: `C` is both carbon and one hundred, so a rest of `C`
/// is read as the numeral, and a molecule whose second element is carbon,
/// vanadium, iodine, lanthanum or lithium is not distinguished here. That is a
/// bound of this parser and not a bound of the record; the molecular identity is
/// #66 and it does not arrive through this door.
fn looks_molecular(rest: &str) -> bool {
    rest.chars()
        .any(|c| c.is_ascii_uppercase() && !matches!(c, 'I' | 'V' | 'X' | 'L' | 'C'))
}

/// Split on the first occurrence of `sep`, refusing a second one.
///
/// A second separator means the input is not what the convention spells, and
/// taking the first split silently would accept it.
fn split_once_exactly(text: &str, sep: char) -> Result<(&str, &str), Unparseable> {
    let (left, right) = text.split_once(sep).ok_or(Unparseable::MissingStage)?;
    if right.contains(sep) {
        return Err(Unparseable::MalformedStage(right.to_owned()));
    }
    Ok((left, right))
}

/// Take the longest leading element symbol, two characters before one.
///
/// `NI` is nitrogen followed by the numeral one, because `NI` is not a symbol,
/// while `Ni` is nickel. Trying the two character symbol first is what keeps
/// those apart.
fn split_leading_symbol(text: &str) -> Result<(Element, &str), Unparseable> {
    for length in [2usize, 1] {
        if text.len() < length || !text.is_char_boundary(length) {
            continue;
        }
        if let Some(element) = Element::from_symbol(&text[..length]) {
            let rest = &text[length..];
            if looks_molecular(rest) {
                return Err(Unparseable::MoreThanOneElement(text.to_owned()));
            }
            return Ok((element, rest));
        }
    }
    if looks_molecular(text) {
        return Err(Unparseable::MoreThanOneElement(text.to_owned()));
    }
    Err(Unparseable::UnknownElement(text.to_owned()))
}

const ROMAN: [(u8, &str); 9] = [
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

fn number_to_roman(mut n: u8) -> String {
    let mut out = String::new();
    for (value, symbol) in ROMAN {
        while n >= value {
            out.push_str(symbol);
            n -= value;
        }
    }
    out
}

/// Read a roman numeral, refusing anything that is not the canonical spelling of
/// the number it denotes.
///
/// Round tripping through `number_to_roman` is what does the refusing. `IIII`
/// reads as four by a lenient parser and renders back as `IV`, so a round trip
/// test would pass on a value the source never wrote. The one character mistake
/// somebody actually makes here is an extra `I`.
fn roman_to_number(text: &str) -> Option<u8> {
    if text.is_empty() {
        return None;
    }
    let mut total: u16 = 0;
    let mut previous: u16 = 0;
    for c in text.chars().rev() {
        let value: u16 = match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            _ => return None,
        };
        if value < previous {
            total = total.checked_sub(value)?;
        } else {
            total = total.checked_add(value)?;
            previous = value;
        }
    }
    let n = u8::try_from(total).ok()?;
    if n == 0 || number_to_roman(n) != text {
        return None;
    }
    Some(n)
}

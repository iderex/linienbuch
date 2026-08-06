# linienbuch

Every abundance determination, exoplanet atmosphere fit and stellar model rests on oscillator strengths and transition probabilities spread over a dozen databases. NIST usually gives no quantitative uncertainties for line intensities, and where it does they are letter grades nobody can propagate. Its Fe I data go back through Fuhr 1988 to Blackwell 1982 while Kurucz computed with the Cowan code, and where a line is in both, NIST is assumed preferred, a hand decision that then sits in a file nobody unpicks. Worse, the way out of poor line lists is astrophysical oscillator strengths tuned to a reference star whose parameters came from atomic data, a circularity that is known, accepted and documented nowhere. This board is a query layer returning every competing value with source, method, year and numerical uncertainty, plus propagation into the derived quantity.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

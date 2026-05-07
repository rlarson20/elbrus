use elbrus_core::{GenericCost, ManaCost, ManaSymbol};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, value},
    multi::many0,
    sequence::delimited,
};
fn symbol(input: &str) -> IResult<&str, ManaSymbol> {
    delimited(char('{'), symbol_inner, char('}')).parse(input)
}

fn symbol_inner(input: &str) -> IResult<&str, ManaSymbol> {
    alt((
        hybrid_phyrexian, // {W/P} — most specific first
        hybrid,           // {2/W}, {W/U}
        phyrexian,        // {P} alone (less common)
        colored,          // {W} {U} {B} {R} {G}
        variable,         // {X}
        colorless,        // {C}
        snow,             // {S}
        tap,              // {T}
        generic,          // {1}, {12}, {100}
        chaos,            // {CHAOS}
        unknown_fallback,
    ))
    .parse(input)
}

fn hybrid_phyrexian(_input: &str) -> IResult<&str, ManaSymbol> {
    todo!()
}
fn hybrid(_input: &str) -> IResult<&str, ManaSymbol> {
    todo!()
}
fn phyrexian(_input: &str) -> IResult<&str, ManaSymbol> {
    todo!()
}
fn colored(input: &str) -> IResult<&str, ManaSymbol> {
    alt((
        value(ManaSymbol::Colored(elbrus_core::Color::W), tag("W")),
        value(ManaSymbol::Colored(elbrus_core::Color::U), tag("U")),
        value(ManaSymbol::Colored(elbrus_core::Color::B), tag("B")),
        value(ManaSymbol::Colored(elbrus_core::Color::R), tag("R")),
        value(ManaSymbol::Colored(elbrus_core::Color::G), tag("G")),
    ))
    .parse(input)
}

fn variable(input: &str) -> IResult<&str, ManaSymbol> {
    alt((
        value(ManaSymbol::Variable(elbrus_core::VarSym::X), tag("X")),
        value(ManaSymbol::Variable(elbrus_core::VarSym::Y), tag("Y")),
        value(ManaSymbol::Variable(elbrus_core::VarSym::Z), tag("Z")),
    ))
    .parse(input)
}

fn colorless(input: &str) -> IResult<&str, ManaSymbol> {
    value(ManaSymbol::Colorless, tag("C")).parse(input)
}
fn snow(input: &str) -> IResult<&str, ManaSymbol> {
    value(ManaSymbol::Snow, tag("S")).parse(input)
}
fn tap(input: &str) -> IResult<&str, ManaSymbol> {
    value(ManaSymbol::Tap, tag("T")).parse(input)
}
fn generic(input: &str) -> IResult<&str, ManaSymbol> {
    map(nom::character::complete::u32, |n| {
        ManaSymbol::Generic(GenericCost::new(n))
    })
    .parse(input)
}
fn chaos(_input: &str) -> IResult<&str, ManaSymbol> {
    todo!()
}
fn unknown_fallback(_input: &str) -> IResult<&str, ManaSymbol> {
    todo!()
}

pub fn mana_cost(input: &str) -> IResult<&str, ManaCost> {
    map(many0(symbol), |symbols| ManaCost(symbols.into())).parse(input)
}

// pub fn parse(input: &str) -> Result<ManaCost, nom::Err<...>> { ... }

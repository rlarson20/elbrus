use elbrus_core::{Color, GenericCost, ManaCost, ManaSymbol, VarSym};
use smallvec::smallvec;

fn parse(input: &str) -> ManaCost {
    elbrus_parser::mana_cost::mana_cost(input)
        .expect("parse failed")
        .1
}

#[test]
fn parse_all_corpus_costs() {
    let raw = include_str!("data/mana_costs.json");
    let costs: Vec<String> = serde_json::from_str(raw).unwrap();
    let mut failures = vec![];
    for s in &costs {
        if elbrus_parser::mana_cost::mana_cost(s).is_err() {
            failures.push(s.clone());
        }
    }
    assert!(failures.is_empty(), "Failed to parse:\n{:#?}", failures);
}
#[test]
fn colored() {
    assert_eq!(
        parse("{G}"),
        ManaCost(smallvec![ManaSymbol::Colored(Color::G)])
    );
}

#[test]
fn generic() {
    assert_eq!(
        parse("{3}"),
        ManaCost(smallvec![ManaSymbol::Generic(GenericCost::new(3))])
    );
}

#[test]
fn hybrid() {
    assert_eq!(
        parse("{G/U}"),
        ManaCost(smallvec![ManaSymbol::Hybrid(Color::G, Color::U)])
    );
}

#[test]
fn twobrid() {
    assert_eq!(
        parse("{2/W}"),
        ManaCost(smallvec![ManaSymbol::TwoBrid(Color::W)])
    );
}

#[test]
fn phyrexian() {
    assert_eq!(
        parse("{W/P}"),
        ManaCost(smallvec![ManaSymbol::Phyrexian(Color::W)])
    );
}

#[test]
fn variable() {
    assert_eq!(
        parse("{X}{X}"),
        ManaCost(smallvec![
            ManaSymbol::Variable(VarSym::X),
            ManaSymbol::Variable(VarSym::X)
        ])
    );
}

// #[test] TODO
// fn complex() {
//     assert_eq!(parse("{X}{2/W}{W/P}{G}"),);
// }

#![allow(dead_code)]
#![allow(unused_imports)]

use regex::Regex;
use std::error;
use std::fmt;
use std::fs;
use std::io::{self, BufRead, BufReader, Cursor};
use std::iter::Iterator;
use std::iter::{FilterMap, FlatMap};
use std::path;

fn main() {}

#[derive(Debug)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn expected<T>(expected: &str, found: &str) -> Result<T, Box<dyn error::Error>> {
        Err(Box::new(ParseError {
            message: format!("Expected {} found {}", expected, found),
        }))
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "Parse error")
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
struct ParsedStuff<'a, T> {
    i: &'a str, // input
    value: T,
}

type ParseResult<'a, T> = Result<ParsedStuff<'a, T>, Box<dyn error::Error>>;

#[derive(Debug, PartialEq)]
enum Subexpr {
    Int(i32),
    Symbol(String),
    SumInt(String, Box<Subexpr>, Box<Subexpr>),
    MulInt(String, Box<Subexpr>, Box<Subexpr>),
    Sequence(Vec<Subexpr>),
    Keyword(String),
    Literal(String),
    Operator(i32, String),
    BinExpr(Box<Subexpr>, Box<Subexpr>, Box<Subexpr>),
}

fn regex<'a, T>(p: &str, input: &'a str, transform: &dyn Fn(&str) -> T) -> ParseResult<'a, T> {
    let pat = format!(r"^\s*{}", p);
    match Regex::new(&pat).unwrap().find(input) {
        Some(m) => Ok(ParsedStuff {
            i: &input[m.end()..],
            value: transform(&m.as_str().trim()),
        }),
        None => ParseError::expected(p, input),
    }
}

fn int(i: &str) -> ParseResult<Subexpr> {
    regex(r"^\s*\d+", i, &|x| Subexpr::Int(x.parse::<i32>().unwrap()))
}

fn literalp<'a>(expected: &'a str) -> impl Fn(&'a str) -> ParseResult<Subexpr> {
    let f = move |x: &str| Subexpr::Literal(x.to_owned());
    move |i| regex(expected, i, &f)
}

fn opp<'a>(precedence: i32, expected: &'a str) -> impl Fn(&'a str) -> ParseResult<Subexpr> {
    let f = move |x: &str| Subexpr::Operator(precedence, x.to_owned());
    move |i| regex(expected, i, &f)
}

fn keyword<'a, 'e>(expected: &'e str, i: &'a str) -> ParseResult<'a, Subexpr> {
    let f = move |x: &str| Subexpr::Keyword(x.to_owned());
    regex(expected, i, &f)
}

fn keywordp<'a>(expected: &'a str) -> impl Fn(&'a str) -> ParseResult<Subexpr> {
    let f = move |x: &str| Subexpr::Keyword(x.to_owned());
    move |i| regex(expected, i, &f)
}

fn symbol(i: &str) -> ParseResult<Subexpr> {
    regex(r"[a-z_][a-z0-9_]*", i, &|x| Subexpr::Symbol(x.to_owned()))
}

// expr:
//   | plusminus { ... }
//
// plusminus:
//   | plusminus PLUS  timesdiv { ... }
//   | plusminus MINUS timesdiv { ... }
//   | timesdiv                 { ... }
//
// timesdiv:
//   | timesdiv DIV   atom { ... }
//   | timesdiv TIMES atom { ... }
//   | atom                { ... }
//
// atom:
//   | VAR                { ... }
//   | NUM                { ... }
//   | LPAREN expr RPAREN { ... }

fn parse(i: &str) -> ParseResult<Subexpr> {
    env_logger::init();
    expr(i)
}

fn expr(i: &str) -> ParseResult<Subexpr> {
    plusminus(i)
}

extern crate log;
fn plusminus(i: &str) -> ParseResult<Subexpr> {
    println!("plusminus called with i={}", i);
    or(
        i,
        "plusminus",
        vec![
            &|i| binop(i, opp(0, "[+-]"), atom, plusminus),
            &|i| binop(i, opp(1, "[*/]"), atom, plusminus),
            &|i| sequence(i, vec![&atom, &literalp(";")]),
        ],
    )
}

fn atom(i: &str) -> ParseResult<Subexpr> {
    println!("atom      called with i={}", i);
    or(i, "atom", vec![&symbol, &int])
}

fn or<'a>(
    i: &'a str,
    err: &str,
    parsers: Vec<&dyn Fn(&'a str) -> ParseResult<Subexpr>>,
) -> ParseResult<'a, Subexpr> {
    println!("or        called with i={}", i);
    for p in parsers {
        let v = p(i);
        if v.is_ok() {
            return v;
        }
    }

    ParseError::expected(err, i)
}

fn sequence<'a>(
    i: &'a str,
    parsers: Vec<&dyn Fn(&'a str) -> ParseResult<Subexpr>>,
) -> ParseResult<'a, Subexpr> {
    let mut expr = Vec::new();
    let mut input = i;

    for p in parsers {
        let ParsedStuff { i, value } = p(input)?;
        input = i;
        match value {
            Subexpr::Literal(_) => {}                   // discart literals
            Subexpr::Sequence(seq) => expr.extend(seq), // flat sequences
            _ => expr.push(value),
        }
    }

    match expr.len() {
        n if n == 1 => Ok(ParsedStuff {
            i: input,
            value: expr.pop().unwrap(),
        }),
        _ => Ok(ParsedStuff {
            i: input,
            value: Subexpr::Sequence(expr),
        }),
    }
}

fn binop<'a>(
    i: &'a str,
    op: impl Fn(&'a str) -> ParseResult<Subexpr>,
    p1: impl Fn(&'a str) -> ParseResult<Subexpr>,
    p2: impl Fn(&'a str) -> ParseResult<Subexpr>,
) -> ParseResult<'a, Subexpr> {
    println!("binop     called with i={}", i);
    let ParsedStuff { i, value: a } = p1(i)?;
    let ParsedStuff { i, value: op } = op(i)?;
    let ParsedStuff { i, value: b } = p2(i)?;

    return Ok(ParsedStuff {
        i,
        value: Subexpr::BinExpr(Box::new(a), Box::new(op), Box::new(b)),
    });
}

mod test {
    use super::*;

    #[test]
    fn test_int() {
        assert_eq!(
            ParsedStuff {
                i: "",
                value: Subexpr::Int(1)
            },
            int(" 1").unwrap()
        );
    }

    #[test]
    fn test_sequence() {
        let res = sequence("1 1;", vec![&int, &int, &literalp(";")])
            .unwrap()
            .value;
        assert_eq!(
            Subexpr::Sequence(vec![
                Subexpr::Int(1),
                Subexpr::Int(1),
                Subexpr::Literal(";".into())
            ]),
            res
        )
    }

    #[test]
    fn test_or() {
        let res = or("1", "error", vec![&literalp("a"), &int]).unwrap().value;
        assert_eq!(Subexpr::Int(1), res)
    }

    #[test]
    fn test_plusminus1() {
        let res = plusminus("1;").unwrap().value;
        assert_eq!(
            Subexpr::Sequence(vec![Subexpr::Int(1), Subexpr::Literal(";".into())]),
            res
        );
    }

    #[test]
    fn test_plusminus2() {
        let res = plusminus("1 + 1;").unwrap().value;
        assert_eq!(
            Subexpr::Sequence(vec![
                Subexpr::Int(1),
                Subexpr::Operator(0, "+".into()),
                Subexpr::Sequence(vec![Subexpr::Int(1), Subexpr::Literal(";".into())])
            ]),
            res
        );
    }

    #[test]
    fn test_plusminus3() {
        let res = plusminus("a + 1 + b;").unwrap().value;
        assert_eq!(
            Subexpr::Sequence(vec![
                Subexpr::Symbol("a".to_owned()),
                Subexpr::Operator(0, "+".into()),
                Subexpr::Int(1),
                Subexpr::Operator(0, "+".into()),
                Subexpr::Sequence(vec![
                    Subexpr::Symbol("b".into()),
                    Subexpr::Literal(";".into())
                ]),
            ]),
            res
        );
    }

    #[test]
    fn test_atom() {
        let res = atom(" a;").unwrap().value;
        println!("{:?}", res);
    }

    #[test]
    fn test_timesdiv1() {
        let res = expr("1 + 1 + a;").unwrap().value;
        println!("{:?}", res);
    }

    #[test]
    fn test_timesdiv2() {
        let res = expr("1 + 1 * a;").unwrap().value;
        println!("{:?}", res);
    }

    #[test]
    fn test_binop() {
        let plus = literalp(r"\+");
        let res = binop("a + b", plus, atom, atom).unwrap();
        println!("{:?}", res);
    }
}

#![allow(dead_code)]
#![allow(unused_imports)]

use std::fs;
use std::io::{self, BufRead, BufReader, Cursor};
use std::iter::Iterator;
use std::path;

fn main() {}

use std::iter::{FilterMap, FlatMap};

#[derive(Debug, PartialEq)]
enum Expr {
    ValInt(String, i32),
}

use regex::Regex;
use std::error;
use std::fmt;

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
type VoidResult<'a> = Result<&'a str, Box<dyn error::Error>>;

fn regex<'a, T>(p: &str, input: &'a str, transform: fn(capture: &str) -> T) -> ParseResult<'a, T> {
    match Regex::new(p).unwrap().captures(input) {
        Some(t) => {
            let len = t[0].len();
            Ok(ParsedStuff {
                i: &input[len..],
                value: transform(&t[0]),
            })
        }
        None => ParseError::expected(p, input),
    }
}

fn int(i: &str) -> ParseResult<i32> {
    regex(r"\d+", i, |x| x.parse::<i32>().unwrap())
}

fn literal<'a>(expected: &str, i: &'a str) -> VoidResult<'a> {
    regex(expected, i, |_| ()).map(|x| x.i)
}

fn val(i: &str) -> VoidResult {
    literal("val", i)
}

fn equal(i: &str) -> VoidResult {
    literal("=", i)
}

fn semicolon(i: &str) -> VoidResult {
    literal(";", i)
}

fn symbol(i: &str) -> ParseResult<String> {
    regex(r"\w+", i, |x| x.to_owned())
}

fn val_expr(i: &str) -> ParseResult<(String, i32)> {
    let i: &str = val(i)?;
    let ParsedStuff { i, value: symbol } = symbol(i)?;
    let i: &str = equal(i)?;
    let ParsedStuff { i, value } = int(i)?;
    let i = semicolon(i)?;

    Ok(ParsedStuff {
        i,
        value: (symbol, value),
    })
}

// val foo = 10: int;;

mod test {
    use super::*;

    #[test]
    fn test_int() {
        assert_eq!(ParsedStuff { i: "", value: 1 }, int("1").unwrap());
    }

    fn test_val_expr() {
        assert_eq!(
            ParsedStuff {
                i: "",
                value: ("foo".to_owned(), 1),
            },
            val_expr("val foo = 1;").unwrap()
        );
    }
}

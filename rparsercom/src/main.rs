/*
 * RUST PARSER COMBINATORS 
 * This isn't really my work, its actually from 
 * https://bodil.lol/parser-combinators/
 */

// Everthing here gets parsed into a struct that looks like this
#[derive (Clone, Debug, PartialEq, Eq)]
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>
}


// Now for the actual parser
// The simplest form of this is just a function that takes some
// input and returns either the parsed input along with the 
// remainig input, or some indication that the input couldn't 
// be parsed. Concretely, we are going to try and turn a
// string into an element struct, as in
//
// fn(&str) -> Result<(&str, Element), &str>
//
//


// This is a parser that looks at a single letter in a string
fn the_letter_a(input: &str) -> Result<(&str, ()), &str>
{
    match input.chars().next() {
        Some('a') => Ok((&input['a'.len_utf8()..], ())),
        _ => Err(input),
    }
}

// function to built a parser
fn match_literal(expected: &'static str) -> impl Fn(&str) -> Result<(&str, ()), &str>
{
    move |input| match input.get(0..expected.len())
    {
        Some(next) if next == expected => {
            Ok((&input[expected.len()..], ()))  // note that the return here is 'parser-like'
        }
        _ => Err(input),
    }
}


// ==== TESTS ==== //
#[test]
fn literal_parser()
{
    let parse_me = match_literal("Hello me!");
    assert_eq!(
        Ok(("", ())),
        parse_me("Hello me!")
    );

    assert_eq!(
        Ok((" Hello you!", ())),    // don't forget leading space...
        parse_me("Hello me! Hello you!")
    );

    assert_eq!(
        Err("Hello you!"),
        parse_me("Hello you!")
    );
}

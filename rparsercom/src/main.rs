/*
 * Rust parser combinator 
 * From here (https://bodil.lol/parser-combinators/)
 */

#[derive(Clone, Debug, PartialEq, Eq)]
struct Element 
{
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>
}

// Make a trait for the parse result 
type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) ->  ParseResult<'a, Output>;
}

// Implement this trait for any function that matches the
// signature of a parser
impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>
    {
        self(input)
    }
}


/*
 * left combinator
 */
fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    return map(pair(parser1, parser2), |(left, _right)| left);
}

/*
 * right combinator
 */
fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    return map(pair(parser1, parser2), |(_left, right)| right);
}

/*
 * match a literal
 */
fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()>
{
    move |input: &'a str| match input.get(0..expected.len()) 
    {
        Some(next) if next == expected => Ok((&input[expected.len()..], ())),
        _ => Err(input),
    }

    //move |input| match input.get(0..expected.len())
    //{
    //    Some(next) if next == expected => {
    //        Ok((&input[expected.len()..], ()))
    //    }
    //    _ => Err(input)
    //}
}

/*
 * match an identifier
 */
fn identifier(input: &str) -> ParseResult<String>
{
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next()
    {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input),
    }

    while let Some(next) = chars.next() 
    {
        if next.is_alphanumeric() || next == '-' {
            matched.push(next);
        } else
        {
            break;
        }
    }

    let next_index = matched.len();
    return Ok((&input[next_index..], matched));
}


// combinator parser for a pair
// this takes two parsers and combines them into a single parser
fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| 
    {
        parser1.parse(input).and_then(|(next_input, result1)|
        {
            parser2.parse(next_input).map(|(last_input, result2)| (last_input, (result1, result2)))
        })
    }
}

// Map combinator 
// We use this to change the type of the result
// This is kind of like the rust equivalent of a functor
fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |input| 
        parser.parse(input)
        .map(|(next_input, result)| (next_input, map_fn(result)))
}

// One-or-more  (.) combinator
fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |mut input| 
    {
        let mut result = Vec::new();

        if let Ok((next_input, first_item)) = parser.parse(input)
        {
            input = next_input;
            result.push(first_item);
        }
        else
        {
            return Err(input);
        }

        while let Ok((next_input, next_item)) = parser.parse(input)
        {
            input = next_input;
            result.push(next_item);
        }

        return Ok((input, result));
    }
}

// Zero-or-more (*) combinator 
fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>
{

    move |mut input|
    {
        let mut result = Vec::new();

        while let Ok((next_input, next_item)) = parser.parse(input)
        {
            input = next_input;
            result.push(next_item);
        }

        return Ok((input, result));
    }
}

// ================ TESTS ================ //

#[test]
fn identifier_parser() 
{
    assert_eq!(
        Ok(("", "i-am-an-identifier".to_string())),
        identifier("i-am-an-identifier")
    );

    assert_eq!(
        Ok((" entirely an identifier", "not".to_string())),
        identifier("not entirely an identifier")
    );

    assert_eq!(
        Err("!not at all an identifier"),
        identifier("!not at all an identifier")
    );
}


#[test]
fn literal_parser() 
{
    let parse_joe = match_literal("Hello Joe!");

    assert_eq!( Ok(("", ())), parse_joe.parse("Hello Joe!") );

    assert_eq!(
        Ok((" Hello Robert!", ())),         // consume "Hello Joe!", leaving "Hello Robert"
        parse_joe.parse("Hello Joe! Hello Robert!")
    );
    
    assert_eq!(
        Err("Hello Mike!"),
        parse_joe.parse("Hello Mike!")
    );
}

#[test]
fn pair_combinator()
{
    // recall that we are actually trying to parse XML
    let tag_opener = pair(match_literal("<"), identifier);

    assert_eq!(
        Ok(("/>", ((), "my-first-element".to_string()))),
        tag_opener.parse("<my-first-element/>")
    );

    assert_eq!(Err("oops"), tag_opener.parse("oops"));
    assert_eq!(Err("!oops"), tag_opener.parse("<!oops"));
}

#[test]
fn test_right_combinator()
{
    let tag_opener = right(match_literal("<"), identifier);

    assert_eq!(
        Ok(("/>", "test-element".to_string())),
        tag_opener.parse("<test-element/>")
    );

    assert_eq!(Err("oops"), tag_opener.parse("oops"));
    assert_eq!(Err("!oops"), tag_opener.parse("<!oops"));
}

// * and . combinators 
#[test]
fn test_one_or_more_combinator()
{
    let parser = one_or_more(match_literal("ha"));

    assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));     
    assert_eq!(Err("ahah"), parser.parse("ahah"));
    assert_eq!(Err(""), parser.parse(""));
}

#[test]
fn test_zero_or_more_combinator()
{
    let parser = zero_or_more(match_literal("ha"));

    assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
    assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
    assert_eq!(Ok(("", vec![])), parser.parse(""));
}



// parser which can parse just the letter 'a'
fn the_letter_a(input: &str) -> Result<(&str, ()), &str>
{
    match input.chars().next()
    {
        Some('a') => Ok((&input['a'.len_utf8()..], ())),
        _ => Err(input),
    }
}

fn main()
{
    let input = "aaab";
    let res = the_letter_a(input);
    println!("Input : {}", input);
    //println!("First call to the_letter_a()");
    println!("{:?}", res);
}

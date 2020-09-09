/*
 * Rust parser combinator 
 * From here (https://bodil.lol/parser-combinators/)
 */
#![type_length_limit="1137931"]

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
        return self(input);
    }
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

/*
 * left combinator
 * Keep only the left side of a combinator pair
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
 * Keep only the right side of a combinator pair
 */
fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    return map(pair(parser1, parser2), |(_left, right)| right);
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

/*
 * parse any character
 */
fn any_char(input: &str) -> ParseResult<char>
{
    match input.chars().next() 
    {
        Some(next) => Ok((&input[next.len_utf8()..], next)),
        _ => Err(input),
    }
}

/*
 * parse and call a predicate function
 */
fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A>
where
    P: Parser<'a, A>,
    F: Fn(&A) -> bool,
{
    move |input| {
        if let Ok((next_input, value)) = parser.parse(input) 
        {
            if predicate(&value)
            {
                return Ok((next_input, value));
            }
        }
        return Err(input);
    }
}

/*
 * parse any whitespace
 */
fn whitespace_char<'a>() -> impl Parser<'a, char>
{
    return pred(any_char, |c| c.is_whitespace());
}

/*
 * parse zero or more/one or more whitespace 
 */
fn one_or_more_space<'a>() -> impl Parser<'a, Vec<char>>
{
    return one_or_more(whitespace_char());
}

fn zero_or_more_space<'a>() -> impl Parser<'a, Vec<char>>
{
    return zero_or_more(whitespace_char());
}

/*
 * parse a quoted string
 */
fn quoted_string<'a>() -> impl Parser<'a, String>
{
    map(
        right(
            match_literal("\""),
            left(
                zero_or_more(pred(any_char, |c| *c != '"')),
                match_literal("\""),
            ),
        ),
        |chars| chars.into_iter().collect(),
    )
}

// ======== ATTRIBUTES ======== //

// This is now quite easy since we have a pair() combinator for parsing 
// a tuple of values which we can combine with an identifier parser.
fn attribute_pair<'a>() -> impl Parser<'a, (String, String)>
{
    return pair(identifier, right(match_literal("="), quoted_string()));
}

// combine the above with zero_or_more to build a vector of attributes 
fn attributes<'a>() -> impl Parser<'a, Vec<(String, String)>>
{
    return zero_or_more(right(one_or_more_space(), attribute_pair()));
}


// Starting element (or opening tag)
fn element_start<'a>() -> impl Parser<'a, (String, Vec<(String, String)>)>
{
    return right(match_literal("<"), pair(identifier, attributes()));
}

// A complete element (with closing tag)
// TODO : this results in an extremely complicated parse result
//fn single_element<'a>() -> impl Parser<'a, Element>
//{
//    return map(
//        left(element_start(), match_literal("/>")),
//        | (name, attributes) | Element {
//            name,
//            attributes,
//            children: vec![],
//        }
//    );
//}


// ================ TESTS ================ //

#[test]
fn test_identifier_parser() 
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
fn test_literal_parser() 
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
fn test_pair_combinator()
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

#[test]
fn test_predicate_combinator() 
{
    let parser = pred(any_char, |c| *c == 'o');
    assert_eq!(Ok(("mg", 'o')), parser.parse("omg")); // get the 'o' from omg
    assert_eq!(Err("lol"), parser.parse("lol"));
}

// test quoted string parser 
#[test]
fn test_quoted_string_parser()
{
    assert_eq!(
        Ok(("", "Hello Joe!".to_string())),
        quoted_string().parse("\"Hello Joe!\"")
    );
}

// test we can parse a single attribute
#[test] 
fn attribute_parser()
{
    assert_eq!(Ok(("",
                  vec![
                      ("one".to_string(), "1".to_string()),
                      ("two".to_string(), "2".to_string())
                  ]
              )),
              attributes().parse(" one=\"1\" two=\"2\"")
          );
}

// test we can parse a single element
//#[test]
//fn single_element_parser()
//{
//    assert_eq!(
//        Ok(("", Element{
//            name: "div".to_string(),
//            attributes: vec![("class".to_string(), "float".to_string())],
//            children: vec![]
//            }
//        )),
//        single_element().parse("<div class=\"float\"/>")
//    );
//}


// ======== MAIN ======== //
fn main()
{
    let input = "hahaha";
    let res = one_or_more(match_literal("ha"));

    println!("Input : {}", input);
    //println!("{:?}", res);
}

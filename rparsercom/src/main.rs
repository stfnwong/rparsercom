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


// parser whic can parse just the letter 'a'
//fn the_letter_a(input: &str) -> Result<(&str, ()), &str>
//{
//    match input.chars().next()
//    {
//        Some('a') => Ok((&input['a'.len_utf8()..], ())),
//        _ => Err(input),
//    }
//}

fn match_literal(expected: &'static str) -> impl Fn(&str) -> Result<(&str, ()), &str>
{
    move |input| match input.get(0..expected.len())
    {
        Some(next) if next == expected => {
            Ok((&input[expected.len()..], ()))
        }
        _ => Err(input)
    }
}

#[test]
fn literal_parser() 
{
    let parse_joe = match_literal("Hello Joe!");

    assert_eq!( Ok(("", ())), parse_joe("Hello Joe!") );

    assert_eq!(
        Ok((" Hello Robert!", ())), 
        parse_joe("Hello Joe! Hello Robert!")
    );
    
    assert_eq!(
        Err("Hello Mike!"),
        parse_joe("Hello Mike!")
    );
}


fn main()
{
    literal_parser();
    //let input = "aaab";
    //let res = the_letter_a(input);
    //println!("Input : {}", input);
    ////println!("First call to the_letter_a()");
    //println!("{:?}", res);
}

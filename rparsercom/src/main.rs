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


use std::iter::Peekable;
use std::str::Chars;

use crate::data::*;

type Input<'a> = Peekable<Chars<'a>>;

fn parse_number(input : &mut Input) -> Result<String, String> {
    let ds = input.take_while(|x| x.is_numeric()).collect::<String>();
    Ok(ds)
}

fn parse_string(input : &mut Input, is_escape : impl Fn(char) -> bool, end : &[char]) -> Result<String, String> {
    let mut rp = input.clone();

    input.next(); // Get rid of initial quote
   
    let mut ret = vec![];
    let mut escape = None;
    loop {
        match input.next() {
            None => { return Err("String encountered end of input".into()); },
            Some(x) if escape.is_some() && end.contains(&x) => { ret.push(x); escape = None; }
            Some(x) if escape.is_some() && is_escape(x) => { ret.push(x); escape = None; }
            Some(x) if escape.is_some() => { ret.push(escape.unwrap()); escape = None; }
            Some(x) if end.contains(&x) => { break; },
            Some(x) if is_escape(x) => { escape = Some(x); },
            Some(x) => { ret.push(x); },
        }
    }

    Ok(ret.into_iter().collect())
}

#[cfg(test)]
mod test {
    use super::*;



}
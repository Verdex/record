
use std::iter::Peekable;
use std::str::Chars;

use crate::data::*;

// TODO can probably have a Records interator
pub fn parse_records(input : &mut impl Iterator<Item = char>, options : &Options) -> Result<Vec<Record>, String> {
    let mut input = input.peekable();

    let mut records = vec![];
    let mut fields = vec![];
    let mut values = vec![];

    loop { 
        match input.peek() {
            None => todo!(),
            Some(x) if options.record.record_div.contains(&x) => { 
                if values.len() != 0 {
                    let mut vs = std::mem::replace(&mut values, vec![]);
                    fields.push(vs);
                }
                if fields.len() != 0 {
                    let mut fs = std::mem::replace(&mut fields, vec![]);
                    records.push(fs);
                }
            },
            Some(x) if options.record.field_div.contains(&x) => { 
                if values.len() != 0 {
                    let mut vs = std::mem::replace(&mut values, vec![]);
                    fields.push(vs);
                }
            },
            Some(x) if options.preserve_spacing && x.is_whitespace() => { values.push(Value::Space(*x)) },
            Some(x) if x.is_numeric() => { values.push(parse_number(&mut input)); },
            Some(x) if x.is_alphabetic() || *x == '_' => { values.push(parse_symbol(&mut input)); },
            Some(x) if options.allow_strings.is_some() && options.allow_strings.as_ref().unwrap().quote_chars.contains(&x) => 
                match options.allow_strings.as_ref().unwrap() {
                    QuoteOpt { escape_char: None, quote_chars } => { values.push(parse_string(&mut input, |_| false, |x| quote_chars.contains(&x))?); },
                    QuoteOpt { escape_char: Some(escape_char), quote_chars } => { values.push(parse_string(&mut input, |x| x == *escape_char, |x| quote_chars.contains(&x))?); },
                },
            Some(x) => { values.push(Value::Punct(*x)); },
            _ => todo!(),
        }
    }

    Err("todo".into())
}

fn parse_number(input : &mut impl Iterator<Item = char>) -> Value {
    Value::Number(input.take_while(|x| x.is_numeric()).collect())
}

fn parse_symbol(input : &mut impl Iterator<Item = char>) -> Value {
    Value::Symbol(input.take_while(|x| x.is_alphanumeric() || *x == '_').collect())
}

fn parse_string( input : &mut impl Iterator<Item = char> 
               , mut is_escape : impl FnMut(char) -> bool
               , mut is_end : impl FnMut(char) -> bool) 
               -> Result<Value, String> {

    input.next(); // Get rid of initial quote
   
    let mut ret = vec![];
    let mut escape = None;
    loop {
        match input.next() {
            None => { return Err("String encountered end of input".into()); },
            Some(x) if escape.is_some() && is_end(x) => { ret.push(x); escape = None; },
            Some(x) if escape.is_some() && is_escape(x) => { ret.push(x); escape = None; },
            Some(x) if escape.is_some() => { ret.push(escape.unwrap()); ret.push(x); escape = None; },
            Some(x) if is_end(x) => { break; },
            Some(x) if is_escape(x) => { escape = Some(x); },
            Some(x) => { ret.push(x); },
        }
    }

    Ok(Value::String(ret.into_iter().collect()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_string_should_parse_string() {
        let mut input = "'string another'".chars();
        let output = parse_string(&mut input, |_| false, |x| x == '\'').unwrap();
        assert_eq!(output, "string another");
    }

    #[test]
    fn parse_string_should_escape() {
        let mut input = "'string \\\\ \\' another'".chars();
        let output = parse_string(&mut input, |x| x == '\\', |x| x == '\'').unwrap();
        assert_eq!(output, "string \\ ' another");
    }

    #[test]
    fn parse_should_should_drop_escape_for_other() {
        let mut input = "'string \\x another'".chars();
        let output = parse_string(&mut input, |x| x == '\\', |x| x == '\'').unwrap();
        assert_eq!(output, "string \\x another");
    }

}

use std::iter::Peekable;

use crate::data::*;

// TODO can probably have a Records iterator 
pub fn parse_records(input : &mut impl Iterator<Item = char>, options : &Options) -> Result<Vec<Record>, String> {
    let mut input = input.peekable();

    let mut records : Vec<Record> = vec![];
    let mut fields : Vec<Field> = vec![];
    let mut values : Vec<Value> = vec![];

    let mut last_was_endline = false;

    loop { 
        let p = input.peek();
        if options.record.record_div == Div::BlankLine {
            if last_was_endline && p == Some(&options.endline) {
                if values.len() != 0 {
                    let vs = std::mem::replace(&mut values, vec![]);
                    fields.push(field(vs));
                }
                let fs = std::mem::replace(&mut fields, vec![]);
                records.push(record(fs));
                input.next();
                continue;
            }
            else if p == Some(&options.endline) {
                last_was_endline = true;
            }
            else {
                last_was_endline = false;
            }
        }
        match p {
            Some(x) if options.record.record_div == Div::EndLine && *x == options.endline => { 
                if values.len() != 0 {
                    let mut vs = std::mem::replace(&mut values, vec![]);
                    fields.push(field(vs));
                }
                let fs = std::mem::replace(&mut fields, vec![]);
                records.push(record(fs));
                input.next();
            },
            Some(x) if options.record.field_div.contains(&x) => { 
                let vs = std::mem::replace(&mut values, vec![]);
                fields.push(field(vs));
                input.next();
            },
            Some(x) if options.preserve_spacing && x.is_whitespace() => { values.push(Value::Space(*x)); input.next(); },
            Some(x) if x.is_whitespace() => { input.next(); },
            Some(x) if x.is_numeric() => { values.push(parse_number(&mut input)); },
            Some(x) if x.is_alphabetic() || *x == '_' => { values.push(parse_symbol(&mut input)); },
            Some(x) if options.strings.is_some() && options.strings.as_ref().unwrap().quote_chars.contains(&x) => 
                match options.strings.as_ref().unwrap() {
                    QuoteOpt { escape_char: None, quote_chars } => { values.push(parse_string(&mut input, |_| false, |x| quote_chars.contains(&x))?); },
                    QuoteOpt { escape_char: Some(escape_char), quote_chars } => { values.push(parse_string(&mut input, |x| x == *escape_char, |x| quote_chars.contains(&x))?); },
                },
            Some(x) => { values.push(Value::Punct(*x)); input.next(); },
            None => {
                if values.len() != 0 {
                    let vs = std::mem::replace(&mut values, vec![]);
                    fields.push(field(vs));
                }
                if fields.len() != 0 {
                    let fs = std::mem::replace(&mut fields, vec![]);
                    records.push(record(fs));
                }
                break;
            },
        }
    }

    Ok(records)
}

fn take_while(input : &mut Peekable<impl Iterator<Item = char>>, mut p : impl FnMut(char) -> bool) -> String {
    let mut cs = vec![];

    while let Some(c) = input.peek() {
        if p(*c) {
            cs.push(input.next().unwrap());
        }
        else {
            break;
        }
    }

    cs.into_iter().collect()
}

fn parse_number(input : &mut Peekable<impl Iterator<Item = char>>) -> Value {
    Value::Number(take_while(input, |x| x.is_numeric()))
}

fn parse_symbol(input : &mut Peekable<impl Iterator<Item = char>>) -> Value {
    Value::Symbol(take_while(input, |x| x.is_alphanumeric() || x == '_'))
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

fn record(fields : Vec<Field>) -> Record {
    Record { fields }
}

fn field(values : Vec<Value>) -> Field {
    Field { values }
}

#[cfg(test)]
mod test {
    use super::*;

    fn num(input : u32) -> Value {
        Value::Number(format!("{}", input))
    }

    #[test]
    fn parse_records_should_parse_multi_line_records_with_space() {
        let mut input = "1 2\n3 4\n\n\n5 6\n7 8".chars();
        let output = parse_records(&mut input, &Options::default().multi_line_records().field_dividers(&['\n']).preserve_spacing(true)).unwrap();

        assert_eq!(output.len(), 3);
        assert_eq!(output[0].fields.len(), 2);
        assert_eq!(output[0].fields[0].values.len(), 3);
        assert_eq!(output[0].fields[0].values[0], num(1));
        assert_eq!(output[0].fields[0].values[1], Value::Space(' '));
        assert_eq!(output[0].fields[0].values[2], num(2));

        assert_eq!(output[0].fields[1].values.len(), 3);
        assert_eq!(output[0].fields[1].values[0], num(3));
        assert_eq!(output[0].fields[1].values[1], Value::Space(' '));
        assert_eq!(output[0].fields[1].values[2], num(4));

        assert_eq!(output[1].fields.len(), 0);

        assert_eq!(output[2].fields.len(), 2);
        assert_eq!(output[2].fields[0].values.len(), 3);
        assert_eq!(output[2].fields[0].values[0], num(5));
        assert_eq!(output[2].fields[0].values[1], Value::Space(' '));
        assert_eq!(output[2].fields[0].values[2], num(6));

        assert_eq!(output[2].fields[1].values.len(), 3);
        assert_eq!(output[2].fields[1].values[0], num(7));
        assert_eq!(output[2].fields[1].values[1], Value::Space(' '));
        assert_eq!(output[2].fields[1].values[2], num(8));
    }

    #[test]
    fn parse_records_should_parse_multi_line_records_with_multi_blank_lines() {
        let mut input = "1 2\n3 4\n\n\n5 6\n7 8".chars();
        let output = parse_records(&mut input, &Options::default().multi_line_records().field_dividers(&['\n'])).unwrap();

        assert_eq!(output.len(), 3);
        assert_eq!(output[0].fields.len(), 2);
        assert_eq!(output[0].fields[0].values.len(), 2);
        assert_eq!(output[0].fields[0].values[0], num(1));
        assert_eq!(output[0].fields[0].values[1], num(2));

        assert_eq!(output[0].fields[1].values.len(), 2);
        assert_eq!(output[0].fields[1].values[0], num(3));
        assert_eq!(output[0].fields[1].values[1], num(4));

        assert_eq!(output[1].fields.len(), 0);

        assert_eq!(output[2].fields.len(), 2);
        assert_eq!(output[2].fields[0].values.len(), 2);
        assert_eq!(output[2].fields[0].values[0], num(5));
        assert_eq!(output[2].fields[0].values[1], num(6));

        assert_eq!(output[2].fields[1].values.len(), 2);
        assert_eq!(output[2].fields[1].values[0], num(7));
        assert_eq!(output[2].fields[1].values[1], num(8));
    }

    #[test]
    fn parse_records_should_parse_multi_line_records_without_final_blank_line() {
        let mut input = "1 2\n3 4\n\n5 6\n7 8".chars();
        let output = parse_records(&mut input, &Options::default().multi_line_records().field_dividers(&['\n'])).unwrap();

        assert_eq!(output.len(), 2);
        assert_eq!(output[0].fields.len(), 2);
        assert_eq!(output[0].fields[0].values.len(), 2);
        assert_eq!(output[0].fields[0].values[0], num(1));
        assert_eq!(output[0].fields[0].values[1], num(2));

        assert_eq!(output[0].fields[1].values.len(), 2);
        assert_eq!(output[0].fields[1].values[0], num(3));
        assert_eq!(output[0].fields[1].values[1], num(4));

        assert_eq!(output[1].fields.len(), 2);
        assert_eq!(output[1].fields[0].values.len(), 2);
        assert_eq!(output[1].fields[0].values[0], num(5));
        assert_eq!(output[1].fields[0].values[1], num(6));

        assert_eq!(output[1].fields[1].values.len(), 2);
        assert_eq!(output[1].fields[1].values[0], num(7));
        assert_eq!(output[1].fields[1].values[1], num(8));
    }

    #[test]
    fn parse_records_should_parse_multi_line_records() {
        let mut input = "1 2\n3 4\n\n5 6\n7 8\n\n".chars();
        let output = parse_records(&mut input, &Options::default().multi_line_records().field_dividers(&['\n'])).unwrap();

        assert_eq!(output.len(), 2);
        assert_eq!(output[0].fields.len(), 2);
        assert_eq!(output[0].fields[0].values.len(), 2);
        assert_eq!(output[0].fields[0].values[0], num(1));
        assert_eq!(output[0].fields[0].values[1], num(2));

        assert_eq!(output[0].fields[1].values.len(), 2);
        assert_eq!(output[0].fields[1].values[0], num(3));
        assert_eq!(output[0].fields[1].values[1], num(4));

        assert_eq!(output[1].fields.len(), 2);
        assert_eq!(output[1].fields[0].values.len(), 2);
        assert_eq!(output[1].fields[0].values[0], num(5));
        assert_eq!(output[1].fields[0].values[1], num(6));

        assert_eq!(output[1].fields[1].values.len(), 2);
        assert_eq!(output[1].fields[1].values[0], num(7));
        assert_eq!(output[1].fields[1].values[1], num(8));
    }

    #[test]
    fn parse_records_should_parse_single_line_records() {
        let mut input = "1,2,3\n4,5,6".chars();
        let output = parse_records(&mut input, &Options::default().single_line_records()).unwrap();

        assert_eq!(output.len(), 2);
        assert_eq!(output[0].fields.len(), 3);
        assert_eq!(output[0].fields[0].values.len(), 1);
        assert_eq!(output[0].fields[0].values[0], num(1));
        assert_eq!(output[0].fields[1].values.len(), 1);
        assert_eq!(output[0].fields[1].values[0], num(2));
        assert_eq!(output[0].fields[2].values.len(), 1);
        assert_eq!(output[0].fields[2].values[0], num(3));

        assert_eq!(output[1].fields.len(), 3);
        assert_eq!(output[1].fields[0].values.len(), 1);
        assert_eq!(output[1].fields[0].values[0], num(4));
        assert_eq!(output[1].fields[1].values.len(), 1);
        assert_eq!(output[1].fields[1].values[0], num(5));
        assert_eq!(output[1].fields[2].values.len(), 1);
        assert_eq!(output[1].fields[2].values[0], num(6));
    }

    #[test]
    fn parse_records_should_parse_single_line_records_with_blank_lines() {
        let mut input = "1,2,3\n\n4,5,6".chars();
        let output = parse_records(&mut input, &Options::default().single_line_records()).unwrap();

        assert_eq!(output.len(), 3);
        assert_eq!(output[0].fields.len(), 3);
        assert_eq!(output[0].fields[0].values.len(), 1);
        assert_eq!(output[0].fields[0].values[0], num(1));
        assert_eq!(output[0].fields[1].values.len(), 1);
        assert_eq!(output[0].fields[1].values[0], num(2));
        assert_eq!(output[0].fields[2].values.len(), 1);
        assert_eq!(output[0].fields[2].values[0], num(3));

        assert_eq!(output[1].fields.len(), 0);

        assert_eq!(output[2].fields.len(), 3);
        assert_eq!(output[2].fields[0].values.len(), 1);
        assert_eq!(output[2].fields[0].values[0], num(4));
        assert_eq!(output[2].fields[1].values.len(), 1);
        assert_eq!(output[2].fields[1].values[0], num(5));
        assert_eq!(output[2].fields[2].values.len(), 1);
        assert_eq!(output[2].fields[2].values[0], num(6));
    }

    #[test]
    fn parse_string_should_parse_string() {
        let mut input = "'string another'".chars();
        let output = parse_string(&mut input, |_| false, |x| x == '\'').unwrap();
        assert_eq!(output, Value::String("string another".into()));
    }

    #[test]
    fn parse_string_should_escape() {
        let mut input = "'string \\\\ \\' another'".chars();
        let output = parse_string(&mut input, |x| x == '\\', |x| x == '\'').unwrap();
        assert_eq!(output, Value::String("string \\ ' another".into()));
    }

    #[test]
    fn parse_should_should_drop_escape_for_other() {
        let mut input = "'string \\x another'".chars();
        let output = parse_string(&mut input, |x| x == '\\', |x| x == '\'').unwrap();
        assert_eq!(output, Value::String("string \\x another".into()));
    }

}
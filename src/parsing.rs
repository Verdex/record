
use std::iter::Peekable;

use crate::data::*;

pub fn parse_records(input : &mut impl Iterator<Item = char>, options : &Options) -> Result<Entry, String> {
    let mut input = input.peekable();

    let mut records = vec![];
    let mut fields = vec![];
    let mut values = vec![];

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
                    let vs = std::mem::replace(&mut values, vec![]);
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
            Some(x) if options.preserve_spacing && x.is_whitespace() => { values.push(Entry::Value(Value::Space(*x))); input.next(); },
            Some(x) if x.is_whitespace() => { input.next(); },
            Some(x) if x.is_numeric() => { values.push(parse_number(&mut input)); },
            Some(x) if x.is_alphabetic() || *x == '_' => { values.push(parse_symbol(&mut input)); },
            Some(x) if options.strings.is_some() && options.strings.as_ref().unwrap().quote_chars.contains(&x) => 
                match options.strings.as_ref().unwrap() {
                    QuoteOpt { escape_char: None, quote_chars } => { values.push(parse_string(&mut input, |_| false, |x| quote_chars.contains(&x))?); },
                    QuoteOpt { escape_char: Some(escape_char), quote_chars } => { values.push(parse_string(&mut input, |x| x == *escape_char, |x| quote_chars.contains(&x))?); },
                },
            Some(x) => { values.push(Entry::Value(Value::Punct(*x))); input.next(); },
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

    Ok(Entry::List(records))
}

fn record(fields : Vec<Entry>) -> Entry {
    Entry::Record(vec![Entry::List(fields)])
}

fn field(values : Vec<Entry>) -> Entry {
    Entry::Field(vec![Entry::List(values)])
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

fn parse_number(input : &mut Peekable<impl Iterator<Item = char>>) -> Entry {
    Entry::Value(Value::Number(take_while(input, |x| x.is_numeric())))
}

fn parse_symbol(input : &mut Peekable<impl Iterator<Item = char>>) -> Entry {
    Entry::Value(Value::Symbol(take_while(input, |x| x.is_alphanumeric() || x == '_')))
}

fn parse_string( input : &mut impl Iterator<Item = char> 
               , mut is_escape : impl FnMut(char) -> bool
               , mut is_end : impl FnMut(char) -> bool) 
               -> Result<Entry, String> {

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

    Ok(Entry::Value(Value::String(ret.into_iter().collect())))
}

#[cfg(test)]
mod test {
    use super::*;

    use std::collections::HashMap;

    use structuralize::pattern::check::*;
    use structuralize::pattern::data::*;
    use structuralize::pattern::matcher::*;

    fn num(input : u32) -> Value {
        Value::Number(format!("{}", input))
    }

    fn precord(p : Pattern<Value>) -> Pattern<Value> {
        Pattern::Cons { name: "Record".into(), params: vec![p] }
    }

    fn pfield(p : Pattern<Value>) -> Pattern<Value> {
        Pattern::Cons { name: "Field".into(), params: vec![p] }
    }

    fn pexact_list(ps : Vec<Pattern<Value>>) -> Pattern<Value> {
        Pattern::ExactList(ps)
    }

    fn plist_path(ps : Vec<Pattern<Value>>) -> Pattern<Value> {
        Pattern::ListPath(ps)
    }

    fn m<'a>(p : Pattern<Value>, d : &'a Entry) -> Vec<HashMap<Box<str>, &'a Entry>> {
        let tc = check_pattern(p).unwrap();
        pattern_match(&tc, d).map(|x| x.into_iter().collect::<HashMap<_, _>>()).collect::<Vec<_>>()
    }

    fn p_empty_record() -> Pattern<Value> {
            plist_path(vec![precord(pexact_list(vec![]))])
    }

    #[test]
    fn parse_records_should_parse_multi_line_records_with_space() {
        let mut input = "1 2\n3 4\n\n\n5 6\n7 8".chars();
        let output = parse_records(&mut input, &Options::default().multi_line_records().field_dividers(&['\n']).preserve_spacing(true)).unwrap();

        let filled_pattern = plist_path(
            vec![precord(
                pexact_list(vec![Pattern::CaptureVar("line_one".into()), Pattern::CaptureVar("line_two".into())]))]);
        let filled_records = m(filled_pattern, &output);

        assert_eq!(filled_records.len(), 2);

        let filled_internal_pattern = pfield(pexact_list(vec![ Pattern::CaptureVar("a".into())
                                                             , Pattern::CaptureVar("b".into())
                                                             , Pattern::CaptureVar("c".into())
                                                             ]));

        // Filled record 1
        let filled = filled_records[0].get("line_one").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(1)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(Value::Space(' ')));
        assert_eq!(filled_results[0].get("c").unwrap(), &&Entry::Value(num(2)));

        let filled = filled_records[0].get("line_two").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(3)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(Value::Space(' ')));
        assert_eq!(filled_results[0].get("c").unwrap(), &&Entry::Value(num(4)));

        // Filled record 2
        let filled = filled_records[1].get("line_one").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(5)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(Value::Space(' ')));
        assert_eq!(filled_results[0].get("c").unwrap(), &&Entry::Value(num(6)));

        let filled = filled_records[1].get("line_two").unwrap();

        let filled_results = m(filled_internal_pattern, filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(7)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(Value::Space(' ')));
        assert_eq!(filled_results[0].get("c").unwrap(), &&Entry::Value(num(8)));

        // Empty Record
        let empty_records = m(p_empty_record(), &output);
        assert_eq!(empty_records.len(), 1);
    }

    #[test]
    fn parse_records_should_parse_multi_line_records_with_multi_blank_lines() {
        let mut input = "1 2\n3 4\n\n\n5 6\n7 8".chars();
        let output = parse_records(&mut input, &Options::default().multi_line_records().field_dividers(&['\n'])).unwrap();

        let filled_pattern = plist_path(
            vec![precord(
                pexact_list(vec![Pattern::CaptureVar("line_one".into()), Pattern::CaptureVar("line_two".into())]))]);
        let filled_records = m(filled_pattern, &output);

        assert_eq!(filled_records.len(), 2);

        let filled_internal_pattern = pfield(pexact_list(vec![ Pattern::CaptureVar("a".into())
                                                             , Pattern::CaptureVar("b".into())
                                                             ]));

        // Filled record 1
        let filled = filled_records[0].get("line_one").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(1)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(2)));

        let filled = filled_records[0].get("line_two").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(3)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(4)));

        // Filled record 2
        let filled = filled_records[1].get("line_one").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(5)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(6)));

        let filled = filled_records[1].get("line_two").unwrap();

        let filled_results = m(filled_internal_pattern, filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(7)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(8)));

        // Empty Record
        let empty_records = m(p_empty_record(), &output);
        assert_eq!(empty_records.len(), 1);
    }

    #[test]
    fn parse_records_should_parse_multi_line_records_without_final_blank_line() {
        let mut input = "1 2\n3 4\n\n5 6\n7 8".chars();
        let output = parse_records(&mut input, &Options::default().multi_line_records().field_dividers(&['\n'])).unwrap();

        let filled_pattern = plist_path(
            vec![precord(
                pexact_list(vec![Pattern::CaptureVar("line_one".into()), Pattern::CaptureVar("line_two".into())]))]);
        let filled_records = m(filled_pattern, &output);

        assert_eq!(filled_records.len(), 2);

        let filled_internal_pattern = pfield(pexact_list(vec![ Pattern::CaptureVar("a".into())
                                                             , Pattern::CaptureVar("b".into())
                                                             ]));

        // Filled record 1
        let filled = filled_records[0].get("line_one").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(1)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(2)));

        let filled = filled_records[0].get("line_two").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(3)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(4)));

        // Filled record 2
        let filled = filled_records[1].get("line_one").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(5)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(6)));

        let filled = filled_records[1].get("line_two").unwrap();

        let filled_results = m(filled_internal_pattern, filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(7)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(8)));

        // Empty Record
        let empty_records = m(p_empty_record(), &output);
        assert_eq!(empty_records.len(), 0);
    }

    #[test]
    fn parse_records_should_parse_multi_line_records() {
        let mut input = "1 2\n3 4\n\n5 6\n7 8\n\n".chars();
        let output = parse_records(&mut input, &Options::default().multi_line_records().field_dividers(&['\n'])).unwrap();

        let filled_pattern = plist_path(
            vec![precord(
                pexact_list(vec![Pattern::CaptureVar("line_one".into()), Pattern::CaptureVar("line_two".into())]))]);
        let filled_records = m(filled_pattern, &output);

        assert_eq!(filled_records.len(), 2);

        let filled_internal_pattern = pfield(pexact_list(vec![ Pattern::CaptureVar("a".into())
                                                             , Pattern::CaptureVar("b".into())
                                                             ]));

        // Filled record 1
        let filled = filled_records[0].get("line_one").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(1)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(2)));

        let filled = filled_records[0].get("line_two").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(3)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(4)));

        // Filled record 2
        let filled = filled_records[1].get("line_one").unwrap();

        let filled_results = m(filled_internal_pattern.clone(), filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(5)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(6)));

        let filled = filled_records[1].get("line_two").unwrap();

        let filled_results = m(filled_internal_pattern, filled);

        assert_eq!(filled_results.len(), 1);
        assert_eq!(filled_results[0].get("a").unwrap(), &&Entry::Value(num(7)));
        assert_eq!(filled_results[0].get("b").unwrap(), &&Entry::Value(num(8)));

        // Empty Record
        let empty_records = m(p_empty_record(), &output);
        assert_eq!(empty_records.len(), 0);
    }

    #[test]
    fn parse_records_should_parse_single_line_records() {
        let mut input = "1,2,3\n4,5,6".chars();
        let output = parse_records(&mut input, &Options::default().single_line_records()).unwrap();

        let record_pattern = plist_path(vec![
                                precord(pexact_list(vec![ pfield(pexact_list(vec![Pattern::CaptureVar("a".into())]))
                                                        , pfield(pexact_list(vec![Pattern::CaptureVar("b".into())]))
                                                        , pfield(pexact_list(vec![Pattern::CaptureVar("c".into())]))
                                                        ]))]);

        let records = m(record_pattern, &output);

        assert_eq!(records.len(), 2);

        assert_eq!(records[0].get("a").unwrap(), &&Entry::Value(num(1)));
        assert_eq!(records[0].get("b").unwrap(), &&Entry::Value(num(2)));
        assert_eq!(records[0].get("c").unwrap(), &&Entry::Value(num(3)));

        assert_eq!(records[1].get("a").unwrap(), &&Entry::Value(num(4)));
        assert_eq!(records[1].get("b").unwrap(), &&Entry::Value(num(5)));
        assert_eq!(records[1].get("c").unwrap(), &&Entry::Value(num(6)));

        // Empty Record
        let empty_records = m(p_empty_record(), &output);
        assert_eq!(empty_records.len(), 0);
    }

    #[test]
    fn parse_records_should_parse_single_line_records_with_blank_lines() {
        let mut input = "1,2,3\n\n4,5,6".chars();
        let output = parse_records(&mut input, &Options::default().single_line_records()).unwrap();

        let record_pattern = plist_path(vec![
                                precord(pexact_list(vec![ pfield(pexact_list(vec![Pattern::CaptureVar("a".into())]))
                                                        , pfield(pexact_list(vec![Pattern::CaptureVar("b".into())]))
                                                        , pfield(pexact_list(vec![Pattern::CaptureVar("c".into())]))
                                                        ]))]);

        let records = m(record_pattern, &output);

        assert_eq!(records.len(), 2);

        assert_eq!(records[0].get("a").unwrap(), &&Entry::Value(num(1)));
        assert_eq!(records[0].get("b").unwrap(), &&Entry::Value(num(2)));
        assert_eq!(records[0].get("c").unwrap(), &&Entry::Value(num(3)));

        assert_eq!(records[1].get("a").unwrap(), &&Entry::Value(num(4)));
        assert_eq!(records[1].get("b").unwrap(), &&Entry::Value(num(5)));
        assert_eq!(records[1].get("c").unwrap(), &&Entry::Value(num(6)));

        // Empty Record
        let empty_records = m(p_empty_record(), &output);
        assert_eq!(empty_records.len(), 1);
    }

    #[test]
    fn parse_string_should_parse_string() {
        let mut input = "'string another'".chars();
        let output = parse_string(&mut input, |_| false, |x| x == '\'').unwrap();
        assert_eq!(output, Entry::Value(Value::String("string another".into())));
    }

    #[test]
    fn parse_string_should_escape() {
        let mut input = "'string \\\\ \\' another'".chars();
        let output = parse_string(&mut input, |x| x == '\\', |x| x == '\'').unwrap();
        assert_eq!(output, Entry::Value(Value::String("string \\ ' another".into())));
    }

    #[test]
    fn parse_should_should_drop_escape_for_other() {
        let mut input = "'string \\x another'".chars();
        let output = parse_string(&mut input, |x| x == '\\', |x| x == '\'').unwrap();
        assert_eq!(output, Entry::Value(Value::String("string \\x another".into())));
    }

}
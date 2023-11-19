

#[derive(Debug)]
pub(crate) struct QuoteOpt { 
    pub(crate) escape_char : Option<char>, 
    pub(crate) quote_chars : Vec<char> 
}

#[derive(Debug, PartialEq)]
pub(crate) enum Div {
    EndLine,
    BlankLine,
}

#[derive(Debug)]
pub(crate) struct RecordOpt {
    pub(crate) record_div : Div,
    pub(crate) field_div : Vec<char>,
}

#[derive(Debug)]
pub struct Options {
    pub(crate) strings : Option<QuoteOpt>,
    pub(crate) record : RecordOpt,
    pub(crate) preserve_spacing : bool,
    pub(crate) endline : char,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Entry {
    Record(Vec<Entry>),
    Field(Vec<Value>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Symbol(String),
    Number(String),
    Space(char),
    Punct(char),
}

impl Default for Options {
    fn default() -> Self {
        Options { strings: Some(QuoteOpt { escape_char: Some('\\'), quote_chars: vec!['\'', '"'] } ) 
                , preserve_spacing: false
                , endline: '\n' 
                , record: RecordOpt { field_div: vec![','], record_div: Div::EndLine }
                }
    }
}

impl Options {
    pub fn endline(mut self, endline : char) -> Self {
        self.endline = endline;
        self
    }

    pub fn preserve_spacing(mut self, preserve : bool) -> Self {
        self.preserve_spacing = preserve;
        self
    }

    pub fn field_dividers(mut self, dividers : &[char]) -> Self {
        self.record.field_div = dividers.to_vec();
        self
    }
    
    pub fn single_line_records(mut self) -> Self {
        self.record.record_div = Div::EndLine;
        self
    }

    pub fn multi_line_records(mut self) -> Self {
        self.record.record_div = Div::BlankLine;
        self
    }

    pub fn allow_strings(mut self, quotes : &[char]) -> Self {
        self.strings = Some(QuoteOpt { escape_char: None, quote_chars: quotes.to_vec() });
        self
    }

    pub fn allow_strings_with_escape(mut self, quotes : &[char], escape_char : char) -> Self {
        self.strings = Some(QuoteOpt { escape_char: Some(escape_char), quote_chars: quotes.to_vec() });
        self
    }

    pub fn disallow_strings(mut self) -> Self {
        self.strings = None;
        self
    }
}

// TODO make sure that the constructors enforce:
//  * non-conflicting options (for example, string shouldn't conflict with divs and divs shouldn't conflict with each other, etc)

// TODO matchable implementation for Vec<record> (Iterator<Item = Record> ?)
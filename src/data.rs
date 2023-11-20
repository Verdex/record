
use structuralize::pattern::data::*;

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
    List(Vec<Entry>),
    Field(Vec<Entry>),
    Value(Value),
}

impl Entry {
    pub fn from_list<'a>(&'a self) -> Result<&'a [Entry], String> {
        match self {
            Entry::List(xs) => Ok(xs),
            x => Err(format!("Expected Entry::List but found: {:?}", x)),
        }
    }

    pub fn from_record<'a>(&'a self) -> Result<&'a [Entry], String> {
        match self {
            Entry::Record(xs) => Ok(xs),
            x => Err(format!("Expected Entry::Record but found: {:?}", x)),
        }
    }

    pub fn from_field<'a>(&'a self) -> Result<&'a [Entry], String> {
        match self {
            Entry::Field(xs) => Ok(xs),
            x => Err(format!("Expected Entry::Field but found: {:?}", x)),
        }
    }

    pub fn from_value<'a>(&'a self) -> Result<&'a Value, String> {
        match self {
            Entry::Value(x) => Ok(x),
            x => Err(format!("Expected Entry::Value but found: {:?}", x)),
        }
    }
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

impl Matchable for Entry {
    type Atom = Value;
    type Object = ();

    fn kind(&self) -> MatchKind<Self> {
        match self {
            Entry::List(ls) => MatchKind::List(ls),
            Entry::Record(xs) => MatchKind::Cons("Record".into(), xs),
            Entry::Field(values) => MatchKind::Cons("Field".into(), values),
            Entry::Value(value) => MatchKind::Atom(value),
        }
    }

    fn to_pattern(&self) -> Pattern<Self::Atom> {
        match self {
            Entry::Record(xs) => Pattern::Cons { name: "Record".into(), params: xs.iter().map(|x| x.to_pattern()).collect() },
            Entry::Field(values) => Pattern::Cons { name: "Field".into(), params: values.iter().map(|x| x.to_pattern()).collect() },
            Entry::Value(value) => Pattern::Atom(value.clone()),
            Entry::List(l) => Pattern::ExactList(l.iter().map(|x| x.to_pattern()).collect()),
        }
    }
}

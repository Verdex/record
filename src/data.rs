
// TODO see if most of these can be moved to pub(crate)

pub struct QuoteOpt { 
    pub escape_char : Option<char>, 
    pub quote_chars : Vec<char> 
}

#[derive(PartialEq)]
pub enum Div {
    EndLine,
    BlankLine,
}

pub struct RecordOpt {
    pub record_div : Div,
    pub field_div : Vec<char>,
}

pub struct Options {
    pub allow_strings : Option<QuoteOpt>,
    pub record : RecordOpt,
    pub preserve_spacing : bool,
    pub endline : char,
}

pub struct Record(pub Vec<Field>);
pub struct Field(pub Vec<Value>);

pub enum Value {
    String(String),
    Symbol(String),
    Number(String),
    Space(char),
    Punct(char),
}

// TODO constructors for options
// TODO make sure that the constructors enforce:
//  * non-conflicting options (for example, string shouldn't conflict with divs and divs shouldn't conflict with each other, etc)

// TODO matchable implementation for Vec<record>
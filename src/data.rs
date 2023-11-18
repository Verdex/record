
// TODO see if most of these can be moved to pub(crate)

#[derive(Debug)]
pub struct QuoteOpt { 
    pub escape_char : Option<char>, 
    pub quote_chars : Vec<char> 
}

#[derive(Debug, PartialEq)]
pub enum Div {
    EndLine,
    BlankLine,
}

#[derive(Debug)]
pub struct RecordOpt {
    pub record_div : Div,
    pub field_div : Vec<char>,
}

#[derive(Debug)]
pub struct Options {
    pub allow_strings : Option<QuoteOpt>,
    pub record : RecordOpt,
    pub preserve_spacing : bool,
    pub endline : char,
}

#[derive(Debug)]
pub struct Record(pub Vec<Field>);

#[derive(Debug)]
pub struct Field(pub Vec<Value>);

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Symbol(String),
    Number(String),
    Space(char),
    Punct(char),
}

impl Options {

}

// TODO constructors for options
// TODO make sure that the constructors enforce:
//  * non-conflicting options (for example, string shouldn't conflict with divs and divs shouldn't conflict with each other, etc)

// TODO matchable implementation for Vec<record>
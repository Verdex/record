
// TODO see if most of these can be moved to pub(crate)

pub struct QuoteOpt { 
    pub escape_char : Option<char>, 
    pub quote_chars : Vec<char> 
}

pub struct Div(pub Vec<char>);

pub struct RecordOpt {
    pub record_div : Vec<Div>,
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
//  * at least one record_div (field div can be empty)
//  * non-conflicting options (for example, string shouldn't conflict with divs and divs shouldn't conflict with each other, etc)

// TODO matchable implementation for record
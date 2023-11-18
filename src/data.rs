
pub struct QuoteOpt { 
    escape_char : char, 
    quote_chars : Vec<char> 
}

pub enum Div {
    Char(char),
    Chars(char, char),
}

pub struct RecordOpt {
    record_div : Vec<Div>,
    field_div : Vec<Div>,
}

pub struct Options {
    allow_strings : Option<QuoteOpt>,
    record : RecordOpt,
    preserve_spacing : bool,
}

pub struct Record(Vec<Field>);
pub struct Field(Vec<Value>);

pub enum Value {
    String(String),
    Number(String),
    Space(char),
    Punct(char),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArgsNumType {
    Zero,
    One,
    ZeroOrOne,
    OneOrMore,
    Any,
}

impl ArgsNumType {
    pub fn from_parse(required_arg: Option<()>, multi_args: Option<bool>) -> Self {
        if required_arg.is_some() {
            match multi_args {
                Some(true) => Self::OneOrMore,
                Some(false) => unreachable!(),
                None => Self::One,
            }
        } else {
            match multi_args {
                Some(true) => Self::Any,
                Some(false) => Self::ZeroOrOne,
                None => Self::Zero,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionToken {
    LongOption(String), // --help
    ShortOption(char),  // -h
    OldOption(String),  // -help
}

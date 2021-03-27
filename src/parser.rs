use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, none_of, one_of},
    combinator::{map, opt, value},
    multi::{many1, separated_list1},
    sequence::{delimited, tuple},
    IResult,
};

use crate::enum_::{ArgsNumType, OptionToken};

pub fn parse_line(s: &str) -> IResult<&str, Vec<(OptionToken, ArgsNumType)>> {
    separated_list1(char(','), delimited(multispace0, option_block, multispace0))(s)
}

fn option_block(s: &str) -> IResult<&str, (OptionToken, ArgsNumType)> {
    let (s, (_, option, _, required_arg, _, optional_args, _)) = tuple((
        multispace0,
        alt((long_option, old_option, short_option)),
        multispace0,
        opt(required_arg),
        multispace0,
        opt(optional_args),
        multispace0,
    ))(s)?;

    Ok((
        s,
        (option, ArgsNumType::from_parse(required_arg, optional_args)),
    ))
}

fn alpha(s: &str) -> IResult<&str, char> {
    alt((lower, upper))(s)
}

fn lower(s: &str) -> IResult<&str, char> {
    one_of("abcdefghijklmnopqrstuvwxyz")(s)
}

fn upper(s: &str) -> IResult<&str, char> {
    one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ")(s)
}

fn long_option(s: &str) -> IResult<&str, OptionToken> {
    let (s, (_, x, xs)) = tuple((tag("--"), alpha, many1(alt((char('-'), alpha)))))(s)?;
    let option = vec![x]
        .into_iter()
        .chain(xs.into_iter())
        .collect::<String>();

    Ok((s, OptionToken::LongOption(option)))
}

fn old_option(s: &str) -> IResult<&str, OptionToken> {
    let (s, (_, x, xs)) = tuple((char('-'), alpha, many1(alt((char('-'), alpha)))))(s)?;
    let option = vec![x]
        .into_iter()
        .chain(xs.into_iter())
        .collect::<String>();

    Ok((s, OptionToken::OldOption(option)))
}

fn short_option(s: &str) -> IResult<&str, OptionToken> {
    map(tuple((char('-'), alpha)), |(_, x)| {
        OptionToken::ShortOption(x)
    })(s)
}

fn required_arg(s: &str) -> IResult<&str, ()> {
    value((), tuple((upper, many1(upper))))(s)
}

fn optional_args(s: &str) -> IResult<&str, bool> {
    map(delimited(char('['), many1(none_of("[]")), char(']')), |x| {
        x.into_iter().collect::<String>().ends_with("...")
    })(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_long_option() {
        {
            let result = long_option("--with-arg ARG").unwrap();
            assert_eq!(result.0, " ARG");
            assert_eq!(result.1, OptionToken::LongOption("with-arg".into()));
        }
        assert!(long_option("-a").is_err());
        assert!(long_option("-old-long").is_err());
        assert!(long_option("abc").is_err());
    }

    #[test]
    fn test_old_option() {
        {
            let result = old_option("-with-arg ARG").unwrap();
            assert_eq!(result.0, " ARG");
            assert_eq!(result.1, OptionToken::OldOption("with-arg".into()));
        }
        assert!(old_option("--long").is_err());
        assert!(old_option("abc").is_err());
    }

    #[test]
    fn test_short_option() {
        {
            let result = short_option("-a ARG").unwrap();
            assert_eq!(result.0, " ARG");
            assert_eq!(result.1, OptionToken::ShortOption('a'));
        }
        assert!(short_option("--").is_err());
        assert!(short_option("a").is_err());
    }

    #[test]
    fn test_required_arg() {
        {
            let result = required_arg("ARG description").unwrap();
            assert_eq!(result.0, " description");
        }
        assert!(required_arg("arg").is_err());
    }

    #[test]
    fn test_optional_arg() {
        {
            let result = optional_args("[ARG] description").unwrap();
            assert_eq!(result.0, " description");
            assert!(!result.1);
        }
        {
            let result = optional_args("[ARG ...] description").unwrap();
            assert_eq!(result.0, " description");
            assert!(result.1);
        }
        assert!(optional_args("ARG ...").is_err());
        assert!(optional_args("[ARG []...]").is_err());
    }

    #[test]
    fn test_option_block() {
        {
            let result = option_block("--input ARG [ARG ...] description ...").unwrap();
            assert_eq!(result.0, "description ...");
            assert_eq!(result.1 .0, OptionToken::LongOption("input".into()));
            assert_eq!(result.1 .1, ArgsNumType::OneOrMore);
        }
        {
            let result = option_block("-i ARG [ARG ...] description ...").unwrap();
            assert_eq!(result.0, "description ...");
            assert_eq!(result.1 .0, OptionToken::ShortOption('i'));
            assert_eq!(result.1 .1, ArgsNumType::OneOrMore);
        }
        {
            let result = option_block("-input ARG [ARG ...] description ...").unwrap();
            assert_eq!(result.0, "description ...");
            assert_eq!(result.1 .0, OptionToken::OldOption("input".into()));
            assert_eq!(result.1 .1, ArgsNumType::OneOrMore);
        }
    }

    #[test]
    fn test_line_parser() {
        {
            let result = parse_line("  --input ARG, -i ARG Some description.").unwrap();
            assert_eq!(result.0, "Some description.");
            assert_eq!(result.1[0].0, OptionToken::LongOption("input".into()));
            assert_eq!(result.1[0].1, ArgsNumType::One);
            assert_eq!(result.1[1].0, OptionToken::ShortOption('i'));
            assert_eq!(result.1[1].1, ArgsNumType::One);
        }
    }
}

use core::str::FromStr;
use arrayvec::ArrayVec;

use crate::{ast::{CommandArgument, CommandId, CommandMnumonics, GcodeCommand, MajorMinorNumber}, parser::ParserStackAlloc, ArgumentMnumonic, LexResult, LexerStackAlloc, LexerTrait, Rule, StateList, StateListStackAlloc};

const LEXER_SIZE: usize = 5;
pub fn parse(source: &str) -> Result<GcodeCommand, &'static str> {
    let lexer = LexerStackAlloc::<ParseUnion, LEXER_SIZE> {
        rules: [
            &|s| if s.starts_with(" "){ Some(LexResult{poped_chars: 1, result: ParseUnion::None }) } else { None },
            &|s| if s.starts_with("\n"){ Some(LexResult{poped_chars: 1, result: ParseUnion::NL }) } else { None },
            &|s| ArgumentMnumonic::from_str(s).ok().map(|arg| LexResult{poped_chars: 1, result: ParseUnion::ArgId(arg) }),
            &|s| {
                let regex = safe_regex::regex!(br"([GMN]) ?([0-9])(\.[0-9])?.*");
                if let Some((id_slice, int_slice, decimal_slice)) = regex.match_slices(s.as_bytes()) {
                    let id_str = unsafe{core::str::from_utf8_unchecked(id_slice)};
                    let num_str = unsafe{core::str::from_utf8_unchecked(int_slice)};
                    let mut dec_str = "";
                    if decimal_slice.len() > 1{
                        dec_str = unsafe{core::str::from_utf8_unchecked(&decimal_slice[1..])};
                    }
                    let id = CommandMnumonics::from_str(id_str).unwrap_or(CommandMnumonics::Unknown);
                    let int = u16::from_str(num_str).unwrap_or(0);
                    let dec = u16::from_str(dec_str).unwrap_or(0);
                    let command = CommandId {
                        //mnumonic: s.split_at(id_str.len()).0,
                        mnumonic: id,
                        major: int,
                        minor: dec,
                    };
                    Some(LexResult{result: ParseUnion::GCodeCommandId(command), poped_chars: id_slice.len() + int_slice.len() + decimal_slice.len()} )
                }
                else { None }
            },
            &|s| {
                let number_regex = safe_regex::regex!(br"(-+)?([0-9]+)(\.[0-9]+)?.*");
                if let Some((sign_slice, num_slice, decimal_slice)) = number_regex.match_slices(s.as_bytes()) {
                    //let sign_str = unsafe{core::str::from_utf8_unchecked(id_slice)};
                    let num_str = unsafe{core::str::from_utf8_unchecked(num_slice)};
                    let mut dec = 0;
                    if decimal_slice.len() > 1{
                        let dec_str = unsafe{core::str::from_utf8_unchecked(&decimal_slice[1..])};
                        dec = u16::from_str(dec_str).unwrap_or(0);
                    }
                    let neg = match sign_slice { [b'-'] => -1, _ => 1 };
                    let int = neg * i32::from_str(num_str).unwrap_or(0);
                    let number = MajorMinorNumber {
                        major: int,
                        minor: dec,
                    };
                    Some(LexResult{result: ParseUnion::SignedNumber(number), poped_chars: sign_slice.len() + num_slice.len() + decimal_slice.len()} )
                }
                else { None }
            },
        ],
    };
    let parser = ParserStackAlloc::<'_, ParseUnion, ParseTypeId, 4> {
        rules: [
            Rule{id: ParseTypeId::Arg, pattern: &[ParseTypeId::ArgId, ParseTypeId::Number], func: &|data| {
                match data {
                    [ParseUnion::ArgId(id), ParseUnion::SignedNumber(num)] => Some(ParseUnion::Arg(CommandArgument{mnumonic: id.clone(), value: num.clone()})),
                    _ => None,
                }
            }},
            Rule{id: ParseTypeId::ParsedCommand, pattern: &[ParseTypeId::CommandId, ParseTypeId::Arg, ParseTypeId::Arg, ParseTypeId::Arg], func: &|data| {
                match data {
                    [ParseUnion::GCodeCommandId(id), ParseUnion::Arg(a1), ParseUnion::Arg(a2), ParseUnion::Arg(a3)] => {
                        let mut args: ArrayVec<CommandArgument, 3> = Default::default();
                        args.push(a1.clone());
                        args.push(a2.clone());
                        args.push(a3.clone());
                        Some(ParseUnion::GCodeCommand(GcodeCommand{command_id: id.clone(), arguments: args}))
                    },
                    _ => None,
                }
            }},
            Rule{id: ParseTypeId::ParsedCommand, pattern: &[ParseTypeId::CommandId, ParseTypeId::Arg, ParseTypeId::Arg], func: &|data| {
                match data {
                    [ParseUnion::GCodeCommandId(id), ParseUnion::Arg(a1), ParseUnion::Arg(a2)] => {
                        let mut args: ArrayVec<CommandArgument, 3> = Default::default();
                        args.push(a1.clone());
                        args.push(a2.clone());
                        Some(ParseUnion::GCodeCommand(GcodeCommand{command_id: id.clone(), arguments: args}))
                    },
                    _ => None,
                }
            }},
            Rule{id: ParseTypeId::ParsedCommand, pattern: &[ParseTypeId::CommandId, ParseTypeId::Arg], func: &|data| {
                match data {
                    [ParseUnion::GCodeCommandId(id), ParseUnion::Arg(a1)] => {
                        let mut args: ArrayVec<CommandArgument, 3> = Default::default();
                        args.push(a1.clone());
                        Some(ParseUnion::GCodeCommand(GcodeCommand{command_id: id.clone(), arguments: args}))
                    },
                    _ => None,
                }
            }},
        ]
    };

    let mut state = StateListStackAlloc::<ParseUnion, ParseTypeId, 30>::new();
    for lexed in lexer.iter(source) {
        let id = match lexed.result {
            ParseUnion::None => ParseTypeId::NoOp,
            ParseUnion::SignedNumber(_) => ParseTypeId::Number,
            ParseUnion::GCodeCommandId(_) => ParseTypeId::CommandId,
            ParseUnion::Arg(_) => ParseTypeId::Arg,
            ParseUnion::ArgId(_) => ParseTypeId::ArgId,
            ParseUnion::GCodeCommand(_) => ParseTypeId::ParsedCommand,
            ParseUnion::NL => ParseTypeId::NL,
        };
        if id != ParseTypeId::NoOp {
            state.push(id, lexed.result);
        }
    }
    parser.parse(&mut state);
    match state.data.iter().next() {
        Some(ParseUnion::GCodeCommand(command)) => Ok(command.clone()),
        None => Err("Empty gcode state data."),
        _ => Err("No command to be returned."),
    }
    //return Ok(GcodeCommand{command_id: CommandId { mnumonic: CommandMnumonics::Unknown, major: 1, minor: 2 }, arguments: Default::default()});
}

//fn combine_number_rule(data: &mut [ParseUnion]) -> Option<ParseUnion> {
    //match data {
        //[] => None,
        //_ => None,
    //}
//}

#[derive(PartialEq, Clone, Default)]
pub enum ParseUnion {
    #[default]
    None,
    SignedNumber(MajorMinorNumber),
    GCodeCommandId(CommandId),
    ArgId(ArgumentMnumonic),
    Arg(CommandArgument),
    GCodeCommand(GcodeCommand),
    NL,
}

#[derive(PartialEq, Clone, Default)]
pub enum ParseTypeId {
    #[default]
    NoOp,
    Error,
    Number,
    CommandId,
    ArgId,
    Arg,
    ParsedCommand,
    NL,
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_g0x1y2z3() {
        let source = "G0 X1 Y2 Z3";
        let parsed = parse(source);
        if let Err(e) = parsed {
            assert_eq!(e, "");
        }
        assert!(parsed.is_ok());
        //assert_eq!(parsed, ParseUnion::None);
    }
}

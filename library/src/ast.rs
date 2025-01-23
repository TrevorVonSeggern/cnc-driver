use core::str::FromStr;
use arrayvec::ArrayVec;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum CommandMnumonics {
    Unknown = 0,
    #[default]
    G,
    M,
    N,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseCommandMnumonicError;
impl FromStr for CommandMnumonics {
    type Err = ParseCommandMnumonicError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('G') => Ok(CommandMnumonics::G),
            Some('M') => Ok(CommandMnumonics::M),
            Some('N') => Ok(CommandMnumonics::N),
            None => Err(ParseCommandMnumonicError{}),
            _ => Err(ParseCommandMnumonicError{}),

        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum ArgumentMnumonic {
    #[default]
    X,
    Y,
    Z,
    F,
    P,
    R,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseArgMnumonicError;
impl FromStr for ArgumentMnumonic {
    type Err = ParseArgMnumonicError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('X') => Ok(ArgumentMnumonic::X),
            Some('Y') => Ok(ArgumentMnumonic::Y),
            Some('Z') => Ok(ArgumentMnumonic::Z),
            _ => Err(ParseArgMnumonicError{}),

        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct CommandId {
    pub mnumonic: CommandMnumonics,
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct MajorMinorNumber {
    pub major: i32,
    pub minor: u16,
    pub float: f32,
}

#[derive(Clone, Default, PartialEq)]
pub struct CommandArgument {
    pub mnumonic: ArgumentMnumonic,
    pub value: MajorMinorNumber,
}

#[derive(Clone, Default, PartialEq)]
pub struct GcodeCommand {
    pub command_id: CommandId,
    pub arguments: ArrayVec<CommandArgument, 3>
}

//pub trait GetSource<'a> {
    //fn source(&self) -> &'a str;
//}

//impl<'a, T: GetSource<'a>> GetSource<'a> for [T]
//{
    //fn source(&self) -> &'a str {
        //match self.len() {
            //0 => panic!("Unable to create a str ref from an empty array"),
            //1 => self[0].source(),
            //_ => join_str(self[0].source(), self.last().unwrap().source()),
        //}
    //}
//}

#[cfg(test)]
mod test {
    use super::*;

    //#[test]
    //fn test_ast_source() {
        //let src = "G0";
        //let mut ast: Ast = Default::default();
        //ast.source = src;
        //assert_eq!(ast.source(), src);
    //}

    //#[test]
    //fn test_ast_array_one_source() {
        //let ast: Ast = Ast::new(Default::default(), "abcd");
        //assert_eq!([ast].source(), "abcd");
    //}

    //#[test]
    //fn test_ast_array_two_source() {
        //let src = "0123456789";
        //let (left, right) = src.split_at(5);
        //let l: Ast = Ast::new(Default::default(), left);
        //let r: Ast = Ast::new(Default::default(), right);
        //assert_eq!([l, r].source(), "0123456789");
    //}

    //#[test]
    //fn test_ast_array_three_source() {
        //let src = "0123456789";
        //let (left, rest) = src.split_at(2);
        //let (mid, right) = rest.split_at(2);
        //let l: Ast = Ast::new(Default::default(), left);
        //let m: Ast = Ast::new(Default::default(), mid);
        //let r: Ast = Ast::new(Default::default(), right);
        //assert_eq!([l, m, r].source(), "0123456789");
    //}
}

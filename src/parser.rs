// note: much code taken from Michael Gattozzi's
//       https://github.com/mgattozzi/schemers
//
//MIT License
//
//Copyright (c) 2016 Michael Gattozzi
//
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.

use nom::*;

#[derive(Debug, PartialEq, Eq)]
enum Op {
    Primitive(Pr),
    Special(Sp),
    User(String),
}

#[derive(Debug, PartialEq, Eq)]
enum Pr {
    Add, Sub, Mul, Div,
    Cons, Car, Cdr, List
}

#[derive(Debug, PartialEq, Eq)]
enum Sp {
    Begin, Callcc, Define, If, Lambda, Let
}

#[derive(Debug, PartialEq, Eq)]
struct Procedure {
    op: Op,
    args: Vec<String>
}

named!(string<&[u8], String>,
       do_parse!(
           word: ws!(alphanumeric) >>
               (String::from_utf8(word.to_vec()).unwrap())
       )
);

named!(primitive<&[u8], Op>,
       alt!(
           map!(tag!("+"),    |_| Op::Primitive(Pr::Add))  |
           map!(tag!("-"),    |_| Op::Primitive(Pr::Sub))  |
           map!(tag!("*"),    |_| Op::Primitive(Pr::Mul))  |
           map!(tag!("/"),    |_| Op::Primitive(Pr::Div))  |
           map!(tag!("cons"), |_| Op::Primitive(Pr::Cons)) |
           map!(tag!("car"),  |_| Op::Primitive(Pr::Car))  |
           map!(tag!("cdr"),  |_| Op::Primitive(Pr::Cdr))  |
           map!(tag!("list"), |_| Op::Primitive(Pr::List))
       )
);

named!(special<&[u8], Op>,
       alt!(
           map!(tag!("begin"),   |_| Op::Special(Sp::Begin))  |
           map!(tag!("call/cc"), |_| Op::Special(Sp::Callcc)) |
           map!(tag!("define"),  |_| Op::Special(Sp::Define)) |
           map!(tag!("if"),      |_| Op::Special(Sp::If))     |
           map!(tag!("lambda"),  |_| Op::Special(Sp::Lambda)) |
           map!(tag!("let"),     |_| Op::Special(Sp::Let))
       )
);

named!(user<&[u8], Op>,
       do_parse!(
           user_proc: string >>
               (Op::User(user_proc))
       )
);

named!(op<&[u8], Op>,
       alt!(
           ws!(special)   |
           ws!(primitive) |
           ws!(user)
       )
);

named!(procedure<&[u8], Procedure>,
       do_parse!(
           tag!("(") >>
           op_type: op >>
           arguments: ws!(many0!(string)) >>
           tag!(")") >>
           (Procedure { op: op_type, args: arguments })
       )
);

#[test]
fn string_parser() {
    let comp = String::from("hi");

    match string(b"hi") {
        IResult::Done(_, s) => assert_eq!(comp, s),
        _ => panic!("failed to parse string"),
    }

    match string(b"  hi  ") {
        IResult::Done(_, s) => assert_eq!(comp, s),
        _ => panic!("failed to parse string"),
    }

    match string(b"hi    ") {
        IResult::Done(_, s) => assert_eq!(comp, s),
        _ => panic!("failed to parse string"),
    }

    match string(b"    hi") {
        IResult::Done(_, s) => assert_eq!(comp, s),
        _ => panic!("failed to parse string"),
    }
}

#[test]
fn user_op_parser() {
    match user(b"userop") {
        IResult::Done(_, s) => assert_eq!(Op::User(String::from("userop")), s),
        _ => panic!("failed to parse userop"),
    }
}

#[test]
fn primitive_op_parser() {
    match primitive(b"+") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::Add), a),
        _ => panic!("failed to parse primitive"),
    }
    match primitive(b"-") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::Sub), a),
        _ => panic!("failed to parse primitive"),
    }
    match primitive(b"*") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::Mul), a),
        _ => panic!("failed to parse primitive"),
    }
    match primitive(b"/") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::Div), a),
        _ => panic!("failed to parse primitive"),
    }
    match primitive(b"cons") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::Cons), a),
        _ => panic!("failed to parse primitive"),
    }
    match primitive(b"car") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::Car), a),
        _ => panic!("failed to parse primitive"),
    }
    match primitive(b"cdr") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::Cdr), a),
        _ => panic!("failed to parse primitive"),
    }
    match primitive(b"list") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::List), a),
        _ => panic!("failed to parse primitive"),
    }
}

#[test]
fn special_op_parser() {
    match special(b"begin") {
        IResult::Done(_, a) => assert_eq!(Op::Special(Sp::Begin), a),
        _ => panic!("failed to parse special"),
    }
    match special(b"call/cc") {
        IResult::Done(_, a) => assert_eq!(Op::Special(Sp::Callcc), a),
        _ => panic!("failed to parse special"),
    }
    match special(b"define") {
        IResult::Done(_, a) => assert_eq!(Op::Special(Sp::Define), a),
        _ => panic!("failed to parse special"),
    }
    match special(b"if") {
        IResult::Done(_, a) => assert_eq!(Op::Special(Sp::If), a),
        _ => panic!("failed to parse special"),
    }
    match special(b"lambda") {
        IResult::Done(_, a) => assert_eq!(Op::Special(Sp::Lambda), a),
        _ => panic!("failed to parse special"),
    }
    match special(b"let") {
        IResult::Done(_, a) => assert_eq!(Op::Special(Sp::Let), a),
        _ => panic!("failed to parse special"),
    }
}

#[test]
fn op_parser() {
    match op(b"  +  ") {
        IResult::Done(_, a) => assert_eq!(Op::Primitive(Pr::Add), a),
        _ => panic!("failed to parse primitive"),
    }
    match op(b"  let  ") {
        IResult::Done(_, a) => assert_eq!(Op::Special(Sp::Let), a),
        _ => panic!("failed to parse special"),
    }
    match op(b"  myproc  ") {
        IResult::Done(_, a) => assert_eq!(Op::User(String::from("myproc")), a),
        _ => panic!("ailed to parse user op"),
    }
}

#[test]
fn procedure_parser() {
    let procedure_num = Procedure {
        op: Op::Primitive(Pr::Add),
        args: vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]
    };
    let procedure_user = Procedure {
        op: Op::User(String::from("myproc")),
        args: Vec::new()
    };
    match procedure(b"(+ 1 2 3 4)") {
        IResult::Done(_, a) => assert_eq!(procedure_num, a),
        _ => panic!("failed to parse procedure"),
    }
    match procedure(b"(myproc)") {
        IResult::Done(_, a) => assert_eq!(procedure_user, a),
        _ => panic!("failed to parse procedure"),
    }
}

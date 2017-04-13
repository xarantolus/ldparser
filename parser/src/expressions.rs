use std::str;
use whitespace::opt_space;
use utils::{symbol, number};

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    LogicNot,
    Minus,
    BitwiseNot,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    LogicOr,
    LogicAnd,
    BitwiseOr,
    BitwiseAnd,
    Equals,
    NotEquals,
    Lesser,
    Greater,
    LesserOrEquals,
    GreaterOrEquals,
    ShiftRight,
    ShiftLeft,
    Plus,
    Minus,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Ident(String),
    Number(u64),
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        right: Box<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    TernaryOp {
        condition: Box<Expression>,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

named!(value_ident<Expression>, map!(
    map_res!(
        symbol,
        str::from_utf8
    ),
    |x: &str| Expression::Ident(x.into())
));

named!(value_number<Expression>, map!(
    number,
    |x| Expression::Number(x)
));

named!(value_nested<Expression>, delimited!(
    tag!("("),
    wsc!(expression),
    tag!(")")
));

named!(value_call<Expression>, do_parse!(
    func: map_res!(symbol, str::from_utf8)
    >>
    wsc!(tag!("("))
    >>
    args: separated_list!(
        wsc!(tag!(",")),
        expression
    )
    >>
    opt_space
    >>
    tag!(")")
    >>
    (Expression::Call{
        function: func.into(),
        arguments: args
    })
));

named!(pub value<Expression>, alt_complete!(
    value_nested | value_call | value_number | value_ident
));

named!(expr_unary_op<Expression>, do_parse!(
    op: alt_complete!(
        tag!("-") | tag!("!") | tag!("~")
    )
    >>
    opt_space
    >>
    right: expr_level_1
    >>
    (Expression::UnaryOp{
        operator: match op[0] as char {
            '-' => UnaryOperator::Minus,
            '!' => UnaryOperator::LogicNot,
            '~' => UnaryOperator::BitwiseNot,
            _ => panic!("Invalid operator"),
        },
        right: Box::new(right),
    })
));

named!(expr_level_1<Expression>, alt_complete!(
    expr_unary_op | value
));

named!(expr_level_2<Expression>, do_parse!(
    first: expr_level_1
    >>
    fold: fold_many0!(pair!(
            wsc!(alt_complete!(
                tag!("*") | tag!("/") | tag!("%")
            )),
            expr_level_1
        ),
        first,
        |prev, new: (&'a [u8], Expression)| {
            Expression::BinaryOp {
                left: Box::new(prev),
                operator: match new.0[0] as char {
                    '*' => BinaryOperator::Multiply,
                    '/' => BinaryOperator::Divide,
                    '%' => BinaryOperator::Remainder,
                    _ => panic!("Invalid operator"),
                },
                right: Box::new(new.1)
            }
        }
    )
    >>
    (fold)
));

named!(expr_level_3<Expression>, do_parse!(
    first: expr_level_2
    >>
    fold: fold_many0!(pair!(
            wsc!(alt_complete!(
                tag!("+") | tag!("-")
            )),
            expr_level_2
        ),
        first,
        |prev, new: (&'a [u8], Expression)| {
            Expression::BinaryOp {
                left: Box::new(prev),
                operator: match new.0[0] as char {
                    '+' => BinaryOperator::Plus,
                    '-' => BinaryOperator::Minus,
                    _ => panic!("Invalid operator"),
                },
                right: Box::new(new.1)
            }
        }
    )
    >>
    (fold)
));

named!(expr_level_4<Expression>, do_parse!(
    first: expr_level_3
    >>
    fold: fold_many0!(pair!(
            wsc!(alt_complete!(
                tag!("<<") | tag!(">>")
            )),
            expr_level_3
        ),
        first,
        |prev, new: (&'a [u8], Expression)| {
            Expression::BinaryOp {
                left: Box::new(prev),
                operator: match new.0 {
                    b"<<" => BinaryOperator::ShiftLeft,
                    b">>" => BinaryOperator::ShiftRight,
                    _ => panic!("Invalid operator"),
                },
                right: Box::new(new.1)
            }
        }
    )
    >>
    (fold)
));

named!(expr_level_5<Expression>, do_parse!(
    first: expr_level_4
    >>
    fold: fold_many0!(pair!(
            wsc!(alt_complete!(
                tag!("==") | tag!("!=") | tag!("<=") | tag!(">=") | tag!("<") | tag!(">")
            )),
            expr_level_4
        ),
        first,
        |prev, new: (&'a [u8], Expression)| {
            Expression::BinaryOp {
                left: Box::new(prev),
                operator: match new.0 {
                    b"==" => BinaryOperator::Equals,
                    b"!=" => BinaryOperator::NotEquals,
                    b"<=" => BinaryOperator::LesserOrEquals,
                    b">=" => BinaryOperator::GreaterOrEquals,
                    b"<" => BinaryOperator::Lesser,
                    b">" => BinaryOperator::Greater,
                    _ => panic!("Invalid operator"),
                },
                right: Box::new(new.1)
            }
        }
    )
    >>
    (fold)
));

named!(expr_level_6<Expression>, do_parse!(
    first: expr_level_5
    >>
    fold: fold_many0!(
        pair!(wsc!(tag!("&")), expr_level_5),
        first,
        |prev, new: (&'a [u8], Expression)| {
            Expression::BinaryOp {
                left: Box::new(prev),
                operator: BinaryOperator::BitwiseAnd,
                right: Box::new(new.1)
            }
        }
    )
    >>
    (fold)
));

named!(expr_level_7<Expression>, do_parse!(
    first: expr_level_6
    >>
    fold: fold_many0!(
        pair!(wsc!(tag!("|")), expr_level_6),
        first,
        |prev, new: (&'a [u8], Expression)| {
            Expression::BinaryOp {
                left: Box::new(prev),
                operator: BinaryOperator::BitwiseOr,
                right: Box::new(new.1)
            }
        }
    )
    >>
    (fold)
));

named!(expr_level_8<Expression>, do_parse!(
    first: expr_level_7
    >>
    fold: fold_many0!(
        pair!(wsc!(tag!("&&")), expr_level_7),
        first,
        |prev, new: (&'a [u8], Expression)| {
            Expression::BinaryOp {
                left: Box::new(prev),
                operator: BinaryOperator::LogicAnd,
                right: Box::new(new.1)
            }
        }
    )
    >>
    (fold)
));

named!(expr_level_9<Expression>, do_parse!(
    first: expr_level_8
    >>
    fold: fold_many0!(
        pair!(wsc!(tag!("||")), expr_level_8),
        first,
        |prev, new: (&'a [u8], Expression)| {
            Expression::BinaryOp {
                left: Box::new(prev),
                operator: BinaryOperator::LogicOr,
                right: Box::new(new.1)
            }
        }
    )
    >>
    (fold)
));

named!(expr_ternary_op<Expression>, do_parse!(
    cond: expr_level_9
    >>
    wsc!(tag!("?"))
    >>
    left: expression
    >>
    wsc!(tag!(":"))
    >>
    right: expression
    >>
    (Expression::TernaryOp{
        condition: Box::new(cond),
        left: Box::new(left),
        right: Box::new(right),
    })
));

named!(pub expression<Expression>, alt_complete!(
    expr_ternary_op | expr_level_9
));

#[cfg(test)]
mod tests {
    use expressions::expression;

    #[test]
    fn test_ws() {
        let x = b"a ( b ( d , ( 0 ) ) , c )";
        assert_done!(expression(x));
        let y = b"a(b(d,(0)),c)";
        assert_eq!(expression(x), expression(y));
    }

    #[test]
    fn test_ternary() {
        let x = b"a ( b ) ? c ( d ) : e";
        assert_done!(expression(x));
    }

    #[test]
    fn test_logic_or() {
        let x = b"a || b";
        assert_done!(expression(x));
    }
}

use std::{fmt, collections::HashMap};
use crate::lexer::{self, Lexer};

pub struct BinExpr {
    lhs: Expr,
    op: i32,
    rhs: Expr,
}

pub struct CallExpr {
    name: String,
    args: Vec<Expr>,
}

pub enum Expr {
    Str(String),
    Int(i32),
    Bool(bool),
    Var(String),
    Bin(Box<BinExpr>),
    Call(Box<CallExpr>),
    Null,
}

fn get_op_prec(op: i32) -> i32 {
    match op {
        x if x == '*' as i32 => 40,
        x if x == '/' as i32 => 40,
        x if x == '+' as i32 => 20,
        x if x == '-' as i32 => 20,
        _ => -1,
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Expr::Str(x) => write!(f, "StrExpr(\"{}\")", x),
            Expr::Int(x) => write!(f, "IntExpr({})", x),
            Expr::Bool(x) => write!(f, "BoolExpr({})", x),
            Expr::Var(x) => write!(f, "VarExpr({})", x),
            Expr::Bin(x) => write!(f, "{}", x),
            Expr::Call(x) => write!(f, "{}", x),
            Expr::Null => write!(f, "NullExpr()"),
        }
    }
}

impl fmt::Display for BinExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BinExpr({}, Op({}), {})", self.lhs, 
            std::char::from_u32(self.op.try_into()
                .unwrap()).unwrap(), self.rhs)
    }
}

impl BinExpr {
    pub fn new(lhs: Expr, op: i32, rhs: Expr) -> Self {
        Self{lhs: lhs, op: op, rhs: rhs}
    }
}

impl fmt::Display for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = write!(f, "FunCall({}(", self.name);
        for expr in self.args.iter() {
            let _ = write!(f, "{expr}, ");
        }
        write!(f, "))")
    }
}

impl CallExpr {
    pub fn new(name: String, args: Vec<Expr>) -> Self {
        CallExpr{name: name, args: args}
    }
}

pub fn parse_num_expr(lexer: &mut Lexer) -> Option<Expr> {
    lexer.assert_token();
    let expr: Option<Expr> = match lexer.front()
        .get_int_val() {
            Some(x) => Some(Expr::Int(*x)),
            None => None,
        };
    lexer.eat();
    expr
}

pub fn parse_str_expr(lexer: &mut Lexer) -> Option<Expr> {
    lexer.assert_token();
    let expr: Option<Expr> = match lexer.front()
        .get_str_val() {
            Some(x) => Some(Expr::Str(x.to_string())),
            None => None,
        };
    lexer.eat();
    expr
}

pub fn parse_bool_expr(lexer: &mut Lexer) -> Option<Expr> {
    lexer.assert_token();
    let expr: Option<Expr> = match lexer.front()
        .get_bool_val() {
            Some(x) => Some(Expr::Bool(*x)),
            None => None,
        };
    lexer.eat();
    expr
}

pub fn parse_paren_expr(lexer: &mut Lexer) -> Option<Expr> {
    lexer.assert_token_kind('(' as i32);
    lexer.eat(); // eat '('
    let expr: Option<Expr> = parse_expr(lexer);
    lexer.assert_token_kind(')' as i32);
    lexer.eat(); // eat ')'
    expr
}

// reference: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl02.html
pub fn parse_bin_rhs(lexer: &mut Lexer, prec: i32, lhs: Expr) -> Option<Expr> {
    loop {
        if lexer.empty() {return Some(lhs);}
        let bin_op: i32 = lexer.front().kind;
        let op_prec: i32 =  get_op_prec(bin_op);
        if op_prec < prec {return Some(lhs);}
        lexer.eat(); // eat bin_op
        let mut rhs: Expr = match parse_primary_expr(lexer) {
            Some(x) => Some(x),
            None => {
                lexer.error("expected a valid expr"
                    .to_string(), None); None
            }
        }.unwrap();
        if lexer.empty() {
            return Some(Expr::Bin(Box::new(BinExpr::new(lhs, bin_op, rhs))));
        }
        let next_op: i32 = lexer.front().kind;
        let next_prec: i32 = get_op_prec(next_op);
        if op_prec < next_prec {
            rhs = match parse_bin_rhs(lexer, op_prec+1, rhs) {
                Some(x) => Some(x),
                None => {
                    lexer.error("expected a valid expr"
                        .to_string(), None); None
                }
            }.unwrap();
        }
        return Some(Expr::Bin(Box::new(BinExpr::new(lhs, bin_op, rhs))));    
    }
}

pub fn parse_iden(lexer: &mut Lexer) -> Option<Expr> {
    lexer.assert_token();
    let name: String = lexer.front().get_str_val()
        .unwrap().to_string();
    lexer.eat(); // eat name
    if lexer.empty() || !lexer.is_token_kind('(' as i32) {
        return Some(Expr::Var(name));
    }
    lexer.eat(); // eat '('
    let mut args: Vec<Expr> = Vec::new();
    while !lexer.empty() && !lexer.is_token_kind(')' as i32) {
        match parse_expr(lexer) {
            Some(x) => args.push(x),
            None => lexer.error(
                String::from("expected correct expr"), None),
        };
        if !lexer.empty() && 
            lexer.is_token_kind(')' as i32) {break;}
        lexer.assert_token_kind(',' as i32);
        lexer.eat(); // eat ','
    }
    lexer.eat(); // eat ')'
    Some(Expr::Call(Box::new(CallExpr{name: name, args: args})))
}

fn parse_primary_expr(lexer: &mut Lexer) -> Option<Expr> {
    lexer.assert_token();
    match lexer.front().kind {
        lexer::TOK_INT => parse_num_expr(lexer),
        lexer::TOK_STR_LIT => parse_str_expr(lexer),
        lexer::TOK_BOOL => parse_bool_expr(lexer),
        lexer::TOK_IDEN => parse_iden(lexer),
        x if x == '(' as i32 => parse_paren_expr(lexer),
        _ => Some(Expr::Null)   
    }
}

pub fn parse_expr(lexer: &mut Lexer) -> Option<Expr> {
    match parse_primary_expr(lexer) {
        Some(x) => parse_bin_rhs(lexer, 0, x),
        None => None,
    }
}

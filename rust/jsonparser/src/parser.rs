use std::io::ErrorKind::Other;

use crate::{JSONValue, lexer::Token::{self, Comma}};

pub struct Parser{
    tokens : Vec<Token>,
    pos : usize
}

impl Parser{
    pub fn new(tokens :Vec<Token>)->Parser{
        return Parser{
            tokens,
            pos:0
        }
    }

    fn current(&self)->&Token {
        if(self.pos<self.tokens.len()){
            return &self.tokens[self.pos]
        }else{
             {panic!("Unexpected end of token stream")}
        }
    }

    fn advance(&mut self){
        self.pos = self.pos+1;
    }

     fn peek(&self)->&Token {
        if(self.pos+1<self.tokens.len()){
            return &self.tokens[self.pos+1]
        }else{
             {panic!("Unexpected end of token stream")}
        }
    }

    fn expect(&mut self) -> &Token {
        if self.pos < self.tokens.len(){
            let output =  &mut self.tokens[self.pos];
            self.pos = self.pos +1;
            return output;
        }else{
             {panic!("Unexpected end of token stream")}
        }
    }

    pub fn parse_object(&mut self)->JSONValue{
        self.advance(); //consume {

        let mut pairs: Vec<(String,JSONValue)> = Vec::new();
        
        if let Token::RightBrace = self.current(){
            self.advance();
            return JSONValue::Object(pairs);
        }

        loop {
            let key = match self.expect(){
                Token::StringToken(s) => s.clone(),
                other => panic!("Expected string key, got something else"),
            };

            //consume colon :

            match self.expect() {
                Token::Colon => {},
                _=>  panic!("Expected colon , got something else"),
            }
            //recursive call
            let value = self.parse_value();

            pairs.push((key,value));

            match self.current() {
                Token::Comma=>{
                    self.advance();
                },
                Token::RightBrace=>{
                    self.advance();
                    break;
                },
                _ => panic!("Expected ',' or '}}' in object"),
            }

        }
        return JSONValue::Object(pairs)

    }

    fn parse_array(&mut self)->JSONValue{
        //consume [
        self.advance();

        let mut elements:Vec<JSONValue> = Vec::new();

        loop{
            let value = self.parse_value();
            elements.push(value);
            match self.current(){
                Token::Comma=>{
                    self.advance();
                }
                Token::RightBracket=>{
                    self.advance();
                    break;
                }
                _=> panic!("issue parsing json value"),
            }
        }

        return JSONValue::Array((elements))
    }

    fn parse_value(&mut self) -> JSONValue{
        match self.current(){
            Token::LeftBrace=>{
                self.parse_object()
            }
            Token::LeftBracket=>{
                self.parse_array()
            }
            Token::False=>{
                self.advance();
                return JSONValue::Bool(true)
            }
            Token::True=>{
                self.advance();
                return JSONValue::Bool(false)
            }
            Token::Null=>{
                self.advance();
                return JSONValue::Null
            }
            Token::NumToken(n)=>{
                let value = *n;
                self.advance();
                JSONValue::Num(value)
            }
            Token::StringToken(s)=>{
                let clone = s.clone();
                self.advance();
                JSONValue::Str(clone)
            }
            Token::RightBrace   => {panic!("Unexpected '`}}`'")}
            Token::RightBracket => {panic!("Unexpected ']'")}
            Token::Colon        => {panic!("Unexpected ':'")}
            Token::Comma        => {panic!("Unexpected ','")}
        }
    }
    }



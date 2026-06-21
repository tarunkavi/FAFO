use std::result;

pub enum Token{
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    StringToken(String),
    NumToken(f64),
    True,
    False,
    Null

}
impl Token{
    pub fn println(&self){
        match self {
            Token::LeftBrace => print!("{{"),
            Token::RightBrace => print!("}}"),
            Token::LeftBracket => print!("["),
            Token::RightBracket => print!("]"),
            Token::Colon => print!(":"),
            Token::Comma => print!(","),
            Token::StringToken(s) => print!("\"{}\"", s),
            Token::NumToken(n) => print!("{}", n),
            Token::True => print!("true"),
            Token::False => print!("false"),
            Token::Null => print!("null"),
        }
    }
}
pub  struct TokenValue{
    token:Token,
    value: String

}

impl TokenValue{
    fn new(token:Token,value:String)->TokenValue{
        return TokenValue { token, value };
    }
}

pub struct Lexer{
    line :Vec<char>,
    pos : usize,
}

impl Lexer{
    pub fn new(l:&str)->Lexer{
        let line = l.chars().collect();
        return Lexer{
            line,
            pos:0
        }
    }

    pub fn current(&self) -> Option<char>{
        if self.pos < self.line.len(){
            Some(self.line[self.pos])
        }else{
            None
        }
    }

    pub fn peek(&self) -> Option<char> {
         if self.pos+1 < self.line.len(){
            Some(self.line[self.pos+1])
        }else{
            None
        }
    }

    pub fn read_literal(&mut self, literal :&str){
        for keyword in literal.chars(){
            match self.current() {
                Some(c) if keyword==c => self.advance(),
                Some(c) => panic!(
                    "Unexpected character '{}' while reading keyword '{}'",
                    c as char, keyword
                ),
                None => panic!("Unexpected end of input while reading keyword '{}'", keyword),
            }
        }
    }



    pub fn read_string(&mut self) -> String{
        let mut s:String = String::new();
        self.advance(); // consume the opening quote
        loop{
            match self.current() {
            None => panic!("Unterminated string"),
            Some('"') =>{
                self.advance();
                return s;
            },
            //can extend this for escaping characters
            Some ('\\') =>{
                    self.advance();
                    match self.current() {
                        Some('"') => {s.push('"');}

                        Some(c) => panic!("Invalid Escape sequence {}",c),
                        None => panic!("Unterminated escape sequence"),

                    }
            }
            Some(c) =>{
                s.push(c);
                self.advance();
            }
            }
           
        }

    }

    pub fn read_number(&mut self)->f64{
        let mut result = String::new();
        match self.current() {
            Some('-')=>{    
                result.push('-');
                self.advance();
            }
            Some('.') => {
                result.push('0');
                result.push('.');
                self.advance();
            }

            _ => {}
        }

        loop{
            match self.current(){
                Some(c @'0'..='9')=>{
                    result.push(c);
                    self.advance();
                }

                Some('.') => {
                result.push('.');
                self.advance();
                }
                

                _ => break,
            }

                
            }

            result.parse::<f64>().unwrap_or_else(|_| panic!("Invalid number: {}", result))

        }





    pub fn advance(&mut self) {
        self.pos += 1;
    }
}
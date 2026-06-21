use core::str;

mod lexer;
mod parser;
use lexer::Token;
use parser::Parser;

enum JSONValue{
    Object(Vec<(String,JSONValue)>),
    Array(Vec<JSONValue>),
    Str(String),
    Num(f64),
    Bool(bool),
    Null,
}

impl std::fmt::Display for JSONValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            JSONValue::Null => write!(f, "null"),
            JSONValue::Bool(b) => write!(f, "{}", b),
            JSONValue::Num(n) => write!(f, "{}", n),
            JSONValue::Str(s) => write!(f, "\"{}\"", s),
            JSONValue::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JSONValue::Object(pairs) => {
                write!(f, "{{")?;
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{}\":{}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

const ERROR_STR: &str = "JSON Parsing error";



fn tokenize(l:&str)->Vec<Token>{
//construct an array of tokens parser will take this array of tokens and then lexical analysis of the rules of the tokens and then finally the parsing.
// array of TK datastructure (Token,value)

let mut lex = lexer::Lexer::new(l);
let mut tokens:Vec<Token> = Vec::new();

loop{
    match lex.current(){
        None => break,

        Some(' ')|Some('\n')|Some('\r')|Some('\t') =>{
            lex.advance();
        }

        Some('{') => {tokens.push(Token::LeftBrace); lex.advance();}
        Some('[') => {tokens.push(Token::LeftBracket);lex.advance();}
        Some(']') => {tokens.push(Token::RightBracket);lex.advance();}
        Some(':') => {tokens.push(Token::Colon);lex.advance();}
        Some(',') => {tokens.push(Token::Comma);lex.advance();}
        Some('}') => {tokens.push(Token::RightBrace);lex.advance();}

        Some('"') =>{
            let s = lex.read_string();
            tokens.push(Token::StringToken(s));
        }
        Some('t') => {
            lex.read_literal("true");
            tokens.push(Token::True);
        }

        Some('f') => {
            lex.read_literal("false");
            tokens.push(Token::False);
        }
        Some('n')=>{
            lex.read_literal("null");
            tokens.push(Token::Null);
        }

        Some('-') | Some('0'..='9')=>{
            let n = lex.read_number();
            tokens.push(Token::NumToken(n));
        }
        // TODO: handle numbers and the true/false/null literals here.
        // For now, skip any other character so the loop terminates.
        Some(c) => {
                panic!("Unexpected character: {}", c as char);
        }
    }

}
 tokens
}

fn parse(tokens:Vec<Token>)->JSONValue{
let mut parser = Parser::new(tokens);
let json = parser.parse_object();

return json
}

fn main() {
// let fp = File::open("src/temp.json").expect("Unable to read the json file");

// let reader = BufReader::new(fp);

// for l in reader.lines(){
//     let line = l.expect("unable to get the line");
//     println!("{}",line);
//     tokenize(line.as_str());
// }
    let input = r#"{"name": "alice", "age": 30.4, "active": true}"#;
    let tokens = tokenize(input);
    let json  = parse(tokens);

    println!("{}", json);

}

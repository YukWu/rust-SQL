use std::{iter::Peekable, str::Chars};
use crate::{error::Result,error::Error};


#[derive(Debug,Clone,PartialEq)]
pub enum Keyword {
    Create,
    Table,
    True,
    False,
    Primary,
    Key,
    Int4,
}

impl Keyword {
    pub fn from_str(ident : &str) -> Option<Self> {
        let r = match ident.to_uppercase().as_ref() {
            "CREATE" => Keyword::Create,
            "TABLE" => Keyword::Table,
            "TRUE" => Keyword::True,
            "FALSE" => Keyword::False,
            "INT" => Keyword::Int4,
            "INTEGER" => Keyword::Int4,
            "PRIMARY" => Keyword::Primary,
            "KEY" => Keyword::Key,
            _ => return None
        };
        Some(r)
    }
}

#[derive(Debug,Clone,PartialEq)]
pub enum Token {
    // 关键字
    Keyword(Keyword),
    // 其他类型的字符串，比如实体名
    Ident(String),
    String(String),
    Number(String),
    OpenParen,
    CloseParen,
    Comma,
    Semicolon,
    Asterisk,
    Plus,
    Minus,
    Slash,
}

pub struct Lexer<'a> {
    iter: Peekable<Chars<'a>>
} 

// 自定义迭代器
impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.scan() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => self.iter.peek().map(|c|Err(Error::Parse(format!("[Lexer] Unexpected character {}",c)))),
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(sql_text: &'a str) -> Self {
        Self {
            iter: sql_text.chars().peekable(),
        }
    }

    fn erase_whitespace(&mut self) {
        self.next_while(|c| c.is_whitespace());
    }

    // 如果满足条件，则指向下一个字符，并返回该字符
    fn next_if<F: Fn(char)->bool> (&mut self,predicate:F) -> Option<char> {
        self.iter.peek().filter(|&c| predicate(*c))?; // 返回None  
        self.iter.next()  // 非 None , 指向下一个字符,并返回当前字符
    }

    fn next_while<F: Fn(char)->bool>(&mut self, predicate:F) -> Option<String> {
        let mut value = String::new();
        while let Some(c) = self.next_if(&predicate) {
            value.push(c);
        }
        Some(value).filter(|v|!v.is_empty())
    }

    // 只有是 Token 类型才跳转到下一个，并返回 Token 
    fn next_if_token<F:Fn(char)->Option<Token>>(&mut self,predicate:F) -> Option<Token> {
        let token = self.iter.peek().and_then(|c|predicate(*c))?;
        self.iter.next();
        Some(token)
    }

    // 获取下一个 token 
    fn scan(&mut self) -> Result<Option<Token>> {
        // 消除字符串中空白字符 
        self.erase_whitespace();
        // 根据第一个字符判断
        match self.iter.peek() {
            Some('\'') => self.scan_string(), // 扫描字符串
            Some(c) if c.is_ascii_digit() => Ok(self.scan_number()),
            Some(c) if c.is_alphabetic() => Ok(self.scan_ident()),
            Some(_) => Ok(self.scan_symbol()),
            None => Ok(None),
        }
    }

    // 扫描字符串
    fn scan_string(&mut self) -> Result<Option<Token>> {
        // 判断是否以单引号开头
        if self.next_if(|c|c=='\'').is_none() {
            return Ok(None);
        }
        
        let mut val = String::new();
        loop {
            match self.iter.next() {
                Some('\'') => break,
                Some(c) => val.push(c),
                None => return Err(Error::Parse(format!("[Lexer] Unexpected end of string"))),
            }
        }
        
        Ok(Some(Token::String(val)))
    }

    // 扫描数字
    fn scan_number(&mut self) -> Option<Token> {
        let mut num = self.next_while(|c|c.is_ascii_digit())?;
        if let Some(sep) = self.next_if(|c|c=='.') {
            num.push(sep);
            // 扫描小数点之后的部分
            while let Some(c) = self.next_if(|c|c.is_ascii_digit()) {
                num.push(c);
            }
        }
        Some(Token::Number(num))
    }

    // 扫描 Ident ，如表名，列名, 也可能是关键字，比如 true / false 
    fn scan_ident(&mut self) -> Option<Token> {
        let mut value = self.next_if(|c|c.is_alphabetic())?.to_string();
        while let Some(c) = self.next_if(|c|c.is_alphanumeric() || c == '_') {
            value.push(c);
        }

        Some(Keyword::from_str(&value).map_or(Token::Ident(value.to_lowercase()),Token::Keyword))
    }

    fn scan_symbol(&mut self) -> Option<Token> {
        self.next_if_token(|c| match c {
            '*' => Some(Token::Asterisk),
            '(' => Some(Token::OpenParen),
            ')' => Some(Token::CloseParen),
            ',' => Some(Token::Comma),
            ';' => Some(Token::Semicolon),
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '/' => Some(Token::Slash),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::Result, sql::parser::lexer::Keyword};

    use super::{Lexer,Token};

    #[test]
    fn test_lexer_create_table() -> Result<()> {
        let tokens = Lexer::new("create table tbl (a int primary key , b integer);").peekable().collect::<Result<Vec<_>>>()?;
        println!("{:?}",tokens);

        assert_eq!(tokens,vec![
            Token::Keyword(Keyword::Create),
            Token::Keyword(Keyword::Table),
            Token::Ident("tbl".to_string()),
            Token::OpenParen,
            Token::Ident("a".to_string()),
            Token::Keyword(Keyword::Int4),
            Token::Keyword(Keyword::Primary),
            Token::Keyword(Keyword::Key),
            Token::Comma,
            Token::Ident("b".to_string()),
            Token::Keyword(Keyword::Int4),
            Token::CloseParen,
            Token::Semicolon
        ]);
        Ok(())
    }
}
use std::{
    char,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    ops::Deref,
    process::exit,
    sync::OnceLock,
};

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Keyword,
    Symbol,
    Identifier,
    IntConst,
    StringConst,
}

#[derive(Debug)]
pub enum Keyword {
    Class,
    Method,
    Function,
    Constructor,
    Int,
    Boolean,
    Char,
    Void,
    Var,
    Static,
    Field,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Null,
    This,
}

#[derive(Debug)]
struct Candidates(Vec<TokenType>);

#[derive(Debug)]
pub struct Tokenizer {
    token: Option<String>,
    source: BufReader<File>,
}

impl Tokenizer {
    fn keywords() -> Option<&'static HashMap<&'static str, Keyword>> {
        static KEYS: OnceLock<HashMap<&str, Keyword>> = OnceLock::new();
        Some(KEYS.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("class", Keyword::Class);
            m.insert("method", Keyword::Method);
            m.insert("function", Keyword::Function);
            m.insert("constructor", Keyword::Constructor);
            m.insert("int", Keyword::Int);
            m.insert("boolean", Keyword::Boolean);
            m.insert("char", Keyword::Char);
            m.insert("void", Keyword::Void);
            m.insert("var", Keyword::Var);
            m.insert("static", Keyword::Static);
            m.insert("field", Keyword::Field);
            m.insert("let", Keyword::Let);
            m.insert("do", Keyword::Do);
            m.insert("if", Keyword::If);
            m.insert("else", Keyword::Else);
            m.insert("while", Keyword::While);
            m.insert("return", Keyword::Return);
            m.insert("true", Keyword::True);
            m.insert("false", Keyword::False);
            m.insert("null", Keyword::Null);
            m.insert("this", Keyword::This);

            m
        }))
    }

    fn symbols() -> &'static String {
        static SYMBOLS: OnceLock<String> = OnceLock::new();
        SYMBOLS.get_or_init(|| "{}()[].,;+-*/&|<>=~".to_string())
    }

    // source: .jack file path
    // if fail to open 'source', exit with 1
    pub fn new(source: &str) -> Self {
        // open source file
        let f = File::open(source).unwrap_or_else(|e| {
            eprintln!("Failed to open source '{}': {e}", source);
            exit(1)
        });

        let reader = BufReader::new(f);

        // construct Tokenizer
        // token points at Before-First
        Self {
            source: reader,
            token: None,
        }
    }

    pub fn has_more_tokens(&mut self) -> bool {
        match self.source.fill_buf() {
            Ok(buf) => !buf.is_empty(),
            Err(_) => false,
        }
    }

    pub fn advance(&mut self) -> Result<(), std::io::Error> {
        let mut token = String::new();
        loop {
            let mut byte = [0u8];
            self.source.read_exact(&mut byte)?;

            let ch = byte[0] as char;

            // continue if head is a whitespace
            if ch.is_ascii_whitespace() {
                continue;
            }

            // return if head is a token
            if Self::symbols().contains(ch) {
                self.token = Some(ch.to_string());
                return Ok(());
            }

            token.push(ch);
            break;
        }

        let buffer = self.source.fill_buf()?;

        token.push_str(
            buffer
                .iter()
                .take_while(|&byte| {
                    !byte.is_ascii_whitespace() && !Self::symbols().contains(char::from(*byte))
                })
                .map(|&b| b as char)
                .collect::<String>()
                .as_str(),
        );

        self.source.consume(token.len() - 1);
        self.token = if token.is_empty() { None } else { Some(token) };

        Ok(())
    }

    pub fn token_type(&self) -> Option<TokenType> {
        let token = self.token.as_deref()?;

        // symbol
        let symbols = Self::symbols();
        if symbols.contains(token) {
            return Some(TokenType::Symbol);
        }

        // keyword
        let keywords = Self::keywords()?;
        if keywords.keys().any(|&t| t == token) {
            return Some(TokenType::Keyword);
        }

        // Integer const
        // 0~32767
        if token.parse::<i32>().is_ok() {
            return Some(TokenType::IntConst);
        }

        // String const
        let begin = token.chars().nth(0)?;
        let last = token.chars().nth_back(0)?;
        if begin == '\"'
            && last == '\"'
            && token.len() >= 2
            && token
                .chars()
                .skip(1)
                .take(token.len() - 2)
                .all(|c| c != '\n' && c != '\"' && c != '\r')
        {
            return Some(TokenType::StringConst);
        }

        // identifier
        // 1. Alphabets
        // 2. Numbers
        // 3. Under score(_)
        // 4. Not start from numbers
        if !begin.is_ascii_digit()
            && token
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_')
        {
            return Some(TokenType::Identifier);
        }

        None
    }

    pub fn keyword(&self) -> Option<&Keyword> {
        if !self.token_type()?.eq(&TokenType::Keyword) {
            return None;
        }

        let keywords = Self::keywords()?;
        let key = self.token.as_deref()?;

        keywords.get(key)
    }

    pub fn symbol(&self) -> Option<&str> {
        self.token.as_deref()
    }
    pub fn identifier(&self) -> Option<&str> {
        self.eq_token_type(&TokenType::Identifier)
    }
    pub fn int_val(&self) -> Option<&str> {
        self.eq_token_type(&TokenType::IntConst)
    }
    pub fn string_val(&self) -> Option<&str> {
        self.eq_token_type(&TokenType::StringConst)
    }

    fn eq_token_type(&self, t: &TokenType) -> Option<&str> {
        if !self.token_type()?.eq(t) {
            return None;
        }

        self.token.as_deref()
    }
}

#[cfg(test)]
mod test {
    use rstest::{fixture, rstest};
    use std::{io::Seek, io::Write};
    use tempfile::NamedTempFile;

    use super::*;

    #[fixture]
    pub fn void_function_main() -> Tokenizer {
        let src = "function void main() {
                            var Array a;
                            var int length;
                            var int i;

                            let length = Keyboard.readInt(\"HOW MANY NUMBERS? \");
                            let a = Array.new(length);
                            let i = 0;
                        }";
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}", src).unwrap();

        Tokenizer::new(file.path().to_str().unwrap())
    }

    #[rstest]
    fn test_has_more_token(
        mut void_function_main: Tokenizer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert!(void_function_main.has_more_tokens());

        void_function_main.source.seek(std::io::SeekFrom::End(0))?;

        assert!(!void_function_main.has_more_tokens());

        Ok(())
    }

    #[rstest]
    #[case("function", 1)]
    #[case("void", 2)]
    #[case("main", 3)]
    #[case("(", 4)]
    #[case(")", 5)]
    #[case("{", 6)]
    #[case("var", 7)]
    #[case("Array", 8)]
    #[case("a", 9)]
    #[case(";", 10)]
    fn test_advance(
        mut void_function_main: Tokenizer,
        #[case] token: String,
        #[case] skip: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(void_function_main.token, None);

        for _ in 0..skip {
            void_function_main.advance().unwrap();
        }

        assert_eq!(void_function_main.token, Some(token));

        Ok(())
    }
    //

    #[fixture]
    fn tokenizer_for_token_text() -> Tokenizer {
        let f = NamedTempFile::new().unwrap();
        let reader = BufReader::new(f.into_file());
        Tokenizer {
            source: reader,
            token: None,
        }
    }

    #[rstest]
    fn test_token_type_for_symbol(mut tokenizer_for_token_text: Tokenizer) {
        for keyword in Tokenizer::symbols().chars() {
            tokenizer_for_token_text.token = Some(keyword.to_string());
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::Symbol)
            );
        }
    }

    #[rstest]
    fn test_token_type_for_keyword(mut tokenizer_for_token_text: Tokenizer) {
        for &keyword in Tokenizer::keywords().unwrap().keys() {
            tokenizer_for_token_text.token = Some(keyword.to_string());
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::Keyword)
            );
        }
    }

    #[rstest]
    fn test_token_type_for_int_const(mut tokenizer_for_token_text: Tokenizer) {
        for &keyword in Tokenizer::keywords().unwrap().keys() {
            tokenizer_for_token_text.token = Some(keyword.to_string());
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::Keyword)
            );
        }
    }

    // int const valid
    #[rstest]
    fn test_token_type_for_int_const_valid(mut tokenizer_for_token_text: Tokenizer) {
        let int_const_valid = [0, 32767, 17, 314];
        for i in int_const_valid {
            tokenizer_for_token_text.token = Some(i.to_string());
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::IntConst)
            );
        }
    }

    // int const valid
    #[rstest]
    fn test_token_type_for_int_const_invalid(mut tokenizer_for_token_text: Tokenizer) {
        let int_const_invalid = [32768];
        for i in int_const_invalid {
            tokenizer_for_token_text.token = Some(i.to_string());
            assert_eq!(tokenizer_for_token_text.token_type(), None);
        }
    }

    // string const valid
    #[rstest]
    #[case("\"abcdefg\"")]
    #[case("\"\"")]
    fn test_token_type_for_string_const_valid(
        mut tokenizer_for_token_text: Tokenizer,
        #[case] input: &str,
    ) {
        tokenizer_for_token_text.token = Some(input.to_string());
        assert_eq!(
            tokenizer_for_token_text.token_type(),
            Some(TokenType::StringConst)
        );
    }

    // string const invalid
    #[rstest]
    fn test_token_type_for_string_const_invalid(mut tokenizer_for_token_text: Tokenizer) {
        let string_const = ["\"abc\"defg\"", "\"aaa\naaa\"", "\"aaa\raaa\""];
        for s in string_const {
            tokenizer_for_token_text.token = Some(s.to_string());
            assert_eq!(tokenizer_for_token_text.token_type(), None);
        }
    }

    // identifier valid
    #[rstest]
    fn test_token_type_for_identifier_valid(mut tokenizer_for_token_text: Tokenizer) {
        let identifier = ["main", "CamelCase", "DevideBy10", "snake_case"];
        for id in identifier {
            tokenizer_for_token_text.token = Some(id.to_string());
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::Identifier)
            );
        }
    }

    // identifier invalid
    #[rstest]
    fn test_token_type_for_identifier_invalid(mut tokenizer_for_token_text: Tokenizer) {
        let identifier = ["10Good", "0Bad"];
        for id in identifier {
            tokenizer_for_token_text.token = Some(id.to_string());
            assert_eq!(tokenizer_for_token_text.token_type(), None);
        }
    }
}

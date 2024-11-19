use std::{
    cell::LazyCell,
    collections::HashMap,
    io::Read,
    fs::File,
    ffi::{c_char, CStr}
};

#[repr(C)]
#[derive(Clone)]
pub enum TokenType {
    SEMICOLON,
    IF,
    THEN,
    END,
    REPEAT,
    UNTIL,
    IDENTIFIER,
    ASSIGN,
    READ,
    WRITE,
    LESSTHAN,
    BIGGERTHAN,
    EQUAL,
    PLUS,
    MINUS,
    MULT,
    DIV,
    OPENBRACKET,
    CLOSEDBRACKET,
    NUMBER,
    UNKNOWN,
}

#[repr(C)]
pub struct Token {
    token_type: TokenType,
    string_val: *const c_char,
    num_val: u64,
}

pub struct Tokenizer {
    input: String,
    current_pos: usize,
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        Tokenizer {
            input,
            current_pos: 0,
        }
    }

    fn is_whitespace(ch: char) -> bool {
        ch.is_whitespace()
    }

    fn is_digit(ch: char) -> bool {
        ch.is_digit(10)
    }

    fn is_letter(ch: char) -> bool {
        ch.is_alphabetic()
    }

    fn skip_whitespace(&mut self) {
        while self.current_pos < self.input.len() && 
              Self::is_whitespace(self.input.chars().nth(self.current_pos).unwrap()) {
            self.current_pos += 1;
        }
    }

    fn tokenize_number(&mut self) -> Option<Token> {
        let start = self.current_pos;
        while self.current_pos < self.input.len() && 
              Self::is_digit(self.input.chars().nth(self.current_pos).unwrap()) {
            self.current_pos += 1;
        }
        
        if start != self.current_pos {
            let num_str = &self.input[start..self.current_pos];
            return Some(Token {
                token_type: TokenType::NUMBER,
                string_val: std::ptr::null(),
                num_val: num_str.parse().unwrap_or(0),
            });
        }
        None
    }

    fn tokenize_identifier(&mut self) -> Option<Token> {
        let start = self.current_pos;
        while self.current_pos < self.input.len() && 
              Self::is_letter(self.input.chars().nth(self.current_pos).unwrap())
            {
            self.current_pos += 1;
        }
        
        if start != self.current_pos {
            let id_str = &self.input[start..self.current_pos];
            
            let token_type = match id_str {
                "if" => TokenType::IF,
                "then" => TokenType::THEN,
                "end" => TokenType::END,
                "repeat" => TokenType::REPEAT,
                "until" => TokenType::UNTIL,
                "read" => TokenType::READ,
                "write" => TokenType::WRITE,
                _ => TokenType::IDENTIFIER,
            };

            return Some(Token {
                token_type,
                string_val: std::ptr::null(),
                num_val: 0,
            });
        }
        None
    }

    fn tokenize_symbol(&mut self) -> Option<Token> {
        if self.current_pos >= self.input.len() {
            return None;
        }

        let ch = self.input.chars().nth(self.current_pos).unwrap();
        let token_type = match ch {
            ';' => TokenType::SEMICOLON,
            '<' => TokenType::LESSTHAN,
            '>' => TokenType::BIGGERTHAN,
            '=' => TokenType::EQUAL,
            '+' => TokenType::PLUS,
            '-' => TokenType::MINUS,
            '*' => TokenType::MULT,
            '/' => TokenType::DIV,
            '(' => TokenType::OPENBRACKET,
            ')' => TokenType::CLOSEDBRACKET,
            ':' => {
                // Check for ':='
                if self.current_pos + 1 < self.input.len() && 
                   self.input.chars().nth(self.current_pos + 1).unwrap() == '=' {
                    self.current_pos += 1;
                    TokenType::ASSIGN
                } else {
                    TokenType::UNKNOWN
                }
            }
            _ => TokenType::UNKNOWN,
        };

        self.current_pos += 1;
        Some(Token {
            token_type,
            string_val: std::ptr::null(),
            num_val: 0,
        })
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.current_pos < self.input.len() {
            self.skip_whitespace();

            if self.current_pos >= self.input.len() {
                break;
            }

            if let Some(token) = self.tokenize_number() {
                tokens.push(token);
            } else if let Some(token) = self.tokenize_identifier() {
                tokens.push(token);
            } else if let Some(token) = self.tokenize_symbol() {
                tokens.push(token);
            } else {
                self.current_pos += 1;
            }
        }

        tokens
    }
}

pub struct CompilationUnit {
    tokens: Vec<String>,
    tokenizer: Option<Tokenizer>,
}

impl CompilationUnit {
    fn open(file_name: &str) -> Self {
        let mut file = File::open(file_name).expect("Unable to open the file");
        let mut code_buffer = String::new();
        file.read_to_string(&mut code_buffer).expect("Unable to read file");
        
        CompilationUnit { 
            tokens: code_buffer.split_whitespace().map(String::from).collect(),
            tokenizer: Some(Tokenizer::new(code_buffer)),
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        self.tokenizer
            .as_mut()
            .expect("Tokenizer not initialized")
            .tokenize()
    }
}

static mut COMPILATION_UNIT: Option<CompilationUnit> = None;

#[no_mangle]
pub extern "C" fn open_compilation(file_name: *const c_char) {
    if file_name.is_null() {
        panic!("No file given");
    }
    
    unsafe {
        let file_name = CStr::from_ptr(file_name).to_str().unwrap();
        if COMPILATION_UNIT.is_none() {
            COMPILATION_UNIT = Some(CompilationUnit::open(file_name));
        }
    }
}

#[no_mangle]
pub extern "C" fn tokenize() -> *mut Token {
    let compilation_unit = unsafe {
        COMPILATION_UNIT.as_mut().expect("No file opened")
    };
    
    let tokens = compilation_unit.tokenize();
    let token_array = tokens.into_boxed_slice();
    let token_ptr = token_array.as_ptr() as *mut Token;
    std::mem::forget(token_array);
    
    token_ptr
}

#[no_mangle]
pub extern "C" fn get_token_count() -> usize {
    unsafe {
        COMPILATION_UNIT
            .as_ref()
            .expect("No file opened")
            .tokens
            .len()
    }
}
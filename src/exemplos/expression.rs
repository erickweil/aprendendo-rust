use std::{collections::HashMap, fmt::{self, write, Error}, io::Result, num::ParseIntError, result, slice};

use rustyline::{error::ReadlineError, history::FileHistory, DefaultEditor, Editor};

use crate::{estruturas::Dir, utils::leia};

#[derive(PartialEq)]
#[derive(Clone)]
enum ExprToken {
    Plus,
    UnaryPlus,
    Minus,
    UnaryMinus,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    BitShiftR,
    BitShiftL,
    Exp,
    Attrib,

    Gt,
    Lt,
    ParO,
    ParC,
    LineNew,
    CarriageReturn,
    Tab,
    Space,

    Word(String),
    LiteralInt(i64),
    LiteralFloat(f64),
}

impl ExprToken {
    fn operator_precedence(&self) -> i32 { // larger means higher precedence
        match &self {
            ExprToken::ParO => 20,
            ExprToken::ParC => 20,
            ExprToken::Exp => 14,
            ExprToken::UnaryPlus => 13,
            ExprToken::UnaryMinus => 13,
            ExprToken::BitNot => 13,
            ExprToken::Mul => 12,
            ExprToken::Div => 12,
            ExprToken::Rem => 12,
            ExprToken::Plus => 11,
            ExprToken::Minus => 11,
            ExprToken::BitShiftR => 10,
            ExprToken::BitShiftL => 10,
            ExprToken::BitAnd => 7,
            ExprToken::BitXor => 6,
            ExprToken::BitOr => 5,
            ExprToken::Attrib => 1,
            _ => -1
        }
    }

    fn is_operator(&self) -> bool {
        self.operator_precedence() >= 0 && *self != ExprToken::ParO && *self != ExprToken::ParC
    }   

    fn get_associativity(&self) -> Dir {
        match &self {
            ExprToken::UnaryPlus => Dir::Right,
            ExprToken::UnaryMinus => Dir::Right,
            ExprToken::BitNot => Dir::Right,
            ExprToken::Exp => Dir::Right,
            _ => Dir::Left
        }
    }

    fn get_number_args(&self) -> i32 {
        match &self {
            ExprToken::UnaryPlus => 1,
            ExprToken::UnaryMinus => 1,
            ExprToken::BitNot => 1,
            ExprToken::Mul => 2,
            ExprToken::Div => 2,
            ExprToken::Rem => 2,
            ExprToken::Plus => 2,
            ExprToken::Minus => 2,
            ExprToken::Exp => 2,
            ExprToken::BitShiftR => 2,
            ExprToken::BitShiftL => 2,
            ExprToken::BitAnd => 2,
            ExprToken::BitXor => 2,
            ExprToken::BitOr => 2,
            ExprToken::Attrib => 2,
            _ => 0         
        }
    }

    fn is_separator(&self) -> bool {
        match &self {
            ExprToken::ParO => true,
            ExprToken::ParC => true,
            ExprToken::Gt => true,
            ExprToken::Lt => true,
            ExprToken::LineNew => true,
            ExprToken::CarriageReturn => true,
            ExprToken::Tab => true,
            ExprToken::Space => true,
            _ => self.is_operator()
        }
    }

    fn is_relevant_token(&self) -> bool {
        match &self {
            ExprToken::LineNew => false,
            ExprToken::CarriageReturn => false,
            ExprToken::Tab => false,
            ExprToken::Space => false,
            _ => true
        }
    }

    fn is_literal(&self) -> bool {
        match self {
            ExprToken::LiteralInt(_) => true,
            ExprToken::LiteralFloat(_) => true,
            _ => false
        }
    }

    fn from_char(c: char) -> Option<ExprToken> {
        match c {
            '+' => Some(ExprToken::Plus),
            '-' => Some(ExprToken::Minus),
            '*' => Some(ExprToken::Mul),
            '/' => Some(ExprToken::Div),
            '%' => Some(ExprToken::Rem),
            '&' => Some(ExprToken::BitAnd),
            '|' => Some(ExprToken::BitOr),
            '^' => Some(ExprToken::BitXor),
            '~' => Some(ExprToken::BitNot),
            '>' => Some(ExprToken::Gt),
            '<' => Some(ExprToken::Lt),
            '(' => Some(ExprToken::ParO),
            ')' => Some(ExprToken::ParC),
            ' ' => Some(ExprToken::Space),
            '\n' => Some(ExprToken::LineNew),
            '\r' => Some(ExprToken::CarriageReturn),
            '\t' => Some(ExprToken::Tab),
            '=' => Some(ExprToken::Attrib),
            _ => None
        }
    }
}

impl fmt::Display for ExprToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self {
            ExprToken::Plus => "+",
            ExprToken::UnaryPlus => "+",
            ExprToken::Minus => "-",
            ExprToken::UnaryMinus => "-",
            ExprToken::Mul => "*",
            ExprToken::Div => "/",
            ExprToken::Rem => "%",
            ExprToken::Exp => "**",
            ExprToken::BitAnd => "&",
            ExprToken::BitOr => "|",
            ExprToken::BitXor => "^",
            ExprToken::BitNot => "~",
            ExprToken::BitShiftR => ">>",
            ExprToken::BitShiftL => "<<",
            ExprToken::Attrib => "=",
            ExprToken::Gt => ">",
            ExprToken::Lt => "<",
            ExprToken::ParO => "(",
            ExprToken::ParC => ")",
            ExprToken::LineNew => "\n",
            ExprToken::CarriageReturn => "\r",
            ExprToken::Tab => "\t",
            ExprToken::Space => " ",
            ExprToken::Word(w) => w,
            ExprToken::LiteralInt(n) => {
                return write!(f, "{}", n);
            },
            ExprToken::LiteralFloat(n) => {
                return write!(f, "{}", n);
            },
            _ => "<?>"
        };
        write!(f, "{}", s)
    }
}

struct Expr(Vec<ExprToken>);
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in self.0.iter() {
            write!(f, "{} ", t);
        }
        writeln!(f)
    }
}

struct ExprParser {}
impl ExprParser {
    fn tokenize(str: &str) -> result::Result<Vec<ExprToken>, String> {
        let mut tokens = Vec::new();
        let mut strbuff = String::new();

        let mut chars = str.chars().peekable();
        while let Some(c) = chars.next() {
            if c >= '0' && c <= '9' { // número
                let mut is_float = false;
                strbuff.clear();
                strbuff.push(c);
                while let Some(&num_c) = chars.peek() {
                    if (num_c >= '0' && num_c <= '9') || num_c == '.' {
                        chars.next();
                        strbuff.push(num_c);
                        if num_c == '.' { is_float = true; }
                    } else {
                        break;
                    }
                }

                if is_float {
                    let result = strbuff.parse::<f64>();
                    if let Ok(num) = result {
                        tokens.push(ExprToken::LiteralFloat(num));
                    } else { 
                        return Err(format!("Número ponto flutuante inválido: '{}'", strbuff));
                    }                
                } else {
                    let result = strbuff.parse::<i64>();
                    if let Ok(num) = result {
                        tokens.push(ExprToken::LiteralInt(num));
                    } else {
                        return Err(format!("Número inteiro inválido: '{}'", strbuff));
                    }
                }
            } else if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' {
                strbuff.clear();
                strbuff.push(c);
                while let Some(&str_c) = chars.peek() {
                    if (str_c >= 'a' && str_c <= 'z') || (str_c >= 'A' && str_c <= 'Z') || (str_c >= '0' && str_c <= '9') || str_c == '_' {
                        chars.next();
                        strbuff.push(str_c);
                    } else {
                        break;
                    }
                }

                tokens.push(ExprToken::Word(strbuff.clone()));
            } else if let Some(t0) = ExprToken::from_char(c) {
                let t1 = ExprToken::from_char(*chars.peek().unwrap_or(&' ')).unwrap_or(ExprToken::Space);

                if t0 == ExprToken::Gt && t1 == ExprToken::Gt {
                    chars.next();
                    tokens.push(ExprToken::BitShiftR);
                } else if t0 == ExprToken::Lt && t1 == ExprToken::Lt {
                    chars.next();
                    tokens.push(ExprToken::BitShiftL);
                } else if t0 == ExprToken::Mul && t1 == ExprToken::Mul {
                    chars.next();
                    tokens.push(ExprToken::Exp);
                } else if t0.is_relevant_token() {
                    tokens.push(t0);
                }            
            } else {
                return Err(format!("Caractere inválido: '{}'", c));
            }
        }

        return Ok(tokens);
    }

    // https://www.andreinc.net/2010/10/05/converting-infix-to-rpn-shunting-yard-algorithm
    // A ideia é implementar análise de expressões
    // de acordo com a notação Reverse Polish Notation
    // Onde '3 1 2 + *' realiza a conta '3 * (1 + 2)'
    /**
        Para todos os tokens:
            Leia o próximo token
            Se é operador(x):
                Enquanto tem um operador(y) na stack
                e:
                    x é associativo para esquerda e a precedência menor ou igual a y
                    ou x e associativo para direita e precedência menor que y

                    remove e printa da stack
                por último insere operador y na stack
            Se é abre parênteses:
                insere token na stack
            Se é fecha parênteses:
                remove tudo da stack e printa até chegar no abre parênteses
                por último descarta o abre parênteses
            Senão:
                printa o token
        
        Por último, remove tudo da stack e printa
    */
    fn convert_to_rpn(expr: &Vec<ExprToken>, expecting_value: &mut bool) -> Vec<ExprToken> {
        let mut output: Vec<ExprToken> = Vec::new();
        let mut stack:  Vec<ExprToken> = Vec::new(); // Pilha com operadores e ( )

        //let mut expecting_value = true;
        for mut elem in expr.clone().into_iter() {
            if elem == ExprToken::ParO {
                stack.push(elem);
                *expecting_value = true;
            } else if elem == ExprToken::ParC {
                while let Some(stackop) = stack.pop() {
                    if stackop == ExprToken::ParO {
                        break;
                    }
                    output.push(stackop);
                }
                *expecting_value = false;
                // stack.pop(); // pop '(' ?
            } else if elem.is_operator() {
                if *expecting_value {
                    if elem == ExprToken::Plus {
                        elem = ExprToken::UnaryPlus;
                    }
                    if elem == ExprToken::Minus {
                        elem = ExprToken::UnaryMinus;                    
                    }
                }

                let elemop_pre = elem.operator_precedence();
                while let Some(stackop) = stack.last() {
                    if *stackop == ExprToken::ParO || *stackop == ExprToken::ParC {
                        break;
                    }
                    let stackop_pre = stackop.operator_precedence();

                    if (elem.get_associativity() == Dir::Left && elemop_pre <= stackop_pre) ||
                    (elem.get_associativity() == Dir::Right && elemop_pre < stackop_pre) {
                        output.push(stack.pop().unwrap())
                    } else {
                        // Se não pare
                        break
                    }
                }

                stack.push(elem);
                *expecting_value = true;
            } else if elem.is_literal() {
                output.push(elem);
                *expecting_value = false;
            } else if let ExprToken::Word(_) = elem {
                output.push(elem);
                *expecting_value = false;
            }
        }

        while let Some(stackop) = stack.pop() {
            output.push(stackop);
        }

        return output;
    }
}

enum EvalValue {
    Literal(f64),
    Variable(String, f64)
}

impl EvalValue {
    fn get(&self) -> f64 {
        match &self {
            EvalValue::Literal(v) => *v,
            EvalValue::Variable(_, v) => *v,
        }
    }
}
struct StackMachine {
    vars: HashMap<String, f64>,
    stack: Vec<EvalValue>
}

impl StackMachine {

    fn new() -> StackMachine {
        StackMachine { 
            vars: HashMap::new(),
            stack: Vec::new()
        }
    }

    fn eval(&mut self, expr: Vec<ExprToken>) -> result::Result<f64, String> {
        self.stack.clear();

        for elem in expr.into_iter() {
            if let ExprToken::Word(name) = elem {
                if let Some((key, value)) = self.vars.get_key_value(&name) {
                    self.stack.push(EvalValue::Variable(name, *value));
                } else {
                    self.vars.insert(name.clone(), 0.0);
                    self.stack.push(EvalValue::Variable(name, 0.0));
                }
            } else if elem.is_literal() {
                let value: f64 = match elem {
                    ExprToken::LiteralFloat(v) => v,
                    ExprToken::LiteralInt(v) => v as f64,
                    _ => 0.0
                };

                self.stack.push(EvalValue::Literal(value));
            } else if elem.is_operator() {
                if elem.get_number_args() == 2 {
                    let a: f64;
                    let b: f64;
                    let mut variable: Option<String> = None;

                    if let Some(v) = self.stack.pop() { b = v.get(); } else {
                        return Err(format!("Faltou argumentos para o operador {}", elem));
                    }
                    if let Some(v) = self.stack.pop() { 
                        a = v.get(); 
                        if let EvalValue::Variable(nome,value) = v {
                            variable = Some(nome);
                        }
                    } else {
                        return Err(format!("Faltou o segundo argumento para o operador {}", elem));
                    }

                    let result = match elem {
                        ExprToken::Attrib => {
                            if variable == None {
                                return Err(format!("O lado esquerdo do operador = deve ser uma variável"));
                            }
                            let variable = variable.unwrap();

                            if let Some(value) = self.vars.get_mut(&variable) {
                                *value = b;

                                *value
                            } else {
                                return Err(format!("Variável '{}' não encontrada", variable));
                            }
                        },
                        ExprToken::Mul => a * b,
                        ExprToken::Div => a / b,
                        ExprToken::Rem => a % b,
                        ExprToken::Plus => a + b,
                        ExprToken::Minus => a - b,
                        ExprToken::Exp => a.powf(b),
                        ExprToken::BitShiftR => ((a as i64) >> (b as i64)) as f64,
                        ExprToken::BitShiftL => ((a as i64) << (b as i64)) as f64,
                        ExprToken::BitAnd => ((a as i64) & (b as i64)) as f64,
                        ExprToken::BitXor => ((a as i64) ^ (b as i64)) as f64,
                        ExprToken::BitOr => ((a as i64) | (b as i64)) as f64,
                        _ => {
                            return Err(format!("Operador '{}' desconhecido", elem));
                        }
                    };
                    self.stack.push(EvalValue::Literal(result));
                } else {
                    let a: f64;
                    if let Some(v) = self.stack.pop() { a = v.get(); } else {
                        return Err(format!("Faltou o argumento para o operador unitário {}", elem));
                    }

                    let result = match elem {
                        ExprToken::UnaryMinus => -a,
                        ExprToken::UnaryPlus => a,
                        ExprToken::BitNot => (!(a as i64)) as f64,
                        _ => {
                            return Err(format!("Operador unitário '{}' desconhecido", elem));
                        }
                    };

                    self.stack.push(EvalValue::Literal(result));
                }
            }
        }

        if let Some(result) = self.stack.pop() {
            return Ok(result.get());
        } else {
            return Err(format!("Não tinha nenhum resultado na pilha"));
        }
    }
}

pub fn expression() {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new().unwrap();

    let mut vm = StackMachine::new();
    let mut prev_input: Option<String> = None;
    loop {
        let mut input: String;
        let result = rl.readline(if prev_input == None { "> " } else { "| " });
        match result {
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                input = line.to_uppercase();
            }
        }

        if let Some(mut prev) = prev_input.take() {
            prev.push_str(&input);
            input = prev;
        }

        if input == "" {
            println!("Você deve escrever uma expressão, ex: 'A = 1 + 2'");
            continue;
        } else if input == "VARS" {
            for (key,value) in vm.vars.iter() {
                println!("{}: {}", key, value);
            }
            continue;
        }

        let result = ExprParser::tokenize(&input);
        if let Err(message) = result {
            println!("Erro: {}", message);
            continue;
        }

        let tokens = result.ok().unwrap();
        let mut expecting_value = true;
        let expr_rpn = ExprParser::convert_to_rpn(&tokens, &mut expecting_value);

        // Para continuar escrevendo na próxima linha
        if expecting_value {
            prev_input = Some(input);
            continue;
        }

        let resultado = vm.eval(expr_rpn);
        if let Ok(value) = resultado {
            println!("{}", value);
        } else if let Err(message) = resultado {
            println!("Erro ao processar: {}", message);
            continue;
        }
    }
}
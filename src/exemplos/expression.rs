use std::{collections::HashMap, fmt::{self, write, Error}, io::Result, num::ParseIntError, result, slice};

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

fn tokenize(str: &str) -> result::Result<Vec<ExprToken>, &str> {
    let mut tokens = Vec::new();
    let mut strbuff = String::new();

    //let mut i = 0;
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
                    if c == '.' { is_float = true; }
                } else {
                    break;
                }
            }

            if is_float {
                let result = strbuff.parse::<f64>();
                if let Ok(num) = result {
                    tokens.push(ExprToken::LiteralFloat(num));
                } else { 
                    return Err("Número ponto flutuante inválido") 
                }                
            } else {
                let result = strbuff.parse::<i64>();
                if let Ok(num) = result {
                    tokens.push(ExprToken::LiteralInt(num));
                } else {
                    return Err("Número inteiro inválido") 
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
            } else if t0.is_relevant_token() {
                tokens.push(t0);
            }            
        } else {
            return Err("Remova o caractere, não deveria estar aqui")
        }

        //i += 1;
    }

    return Ok(tokens);
}

// https://www.andreinc.net/2010/10/05/converting-infix-to-rpn-shunting-yard-algorithm
// A ideia é implementar análise de expressões
// de acordo com a notação Reverse Polish Notation
// Onde '3 1 2 + *' realiza a conta '3 * (1 + 2)'
struct ReversePolish {
}

enum EvalValue<'a> {
    Literal(f64),
    Variable(&'a str, f64)
}

impl<'a> EvalValue<'a> {
    fn get(&self) -> f64 {
        match &self {
            EvalValue::Literal(v) => *v,
            EvalValue::Variable(_, v) => *v,
        }
    }
}

impl ReversePolish {
    fn eval(expr: &Vec<ExprToken>, variables: &mut HashMap<String, f64>) -> result::Result<f64, &'static str> {
        let mut stack: Vec<EvalValue> = Vec::new();
        for elem in expr.iter() {
            if let ExprToken::Word(name) = elem {
                if let Some(value) = variables.get(name) {
                    stack.push(EvalValue::Variable(&name, *value));
                } else {
                    variables.insert(name.to_string(), 0.0);
                    stack.push(EvalValue::Variable(&name, 0.0));
                }
            } else if elem.is_literal() {
                let value: f64 = match elem {
                    ExprToken::LiteralFloat(v) => *v,
                    ExprToken::LiteralInt(v) => *v as f64,
                    _ => 0.0
                };

                stack.push(EvalValue::Literal(value));
            } else if elem.is_operator() {
                if elem.get_number_args() == 2 {
                    let a: f64;
                    let b: f64;
                    let mut variable: Option<&str> = None;

                    if let Some(v) = stack.pop() { b = v.get(); } else {
                        return Err("Faltou argumentos para o operador!!");
                    }
                    if let Some(v) = stack.pop() { 
                        a = v.get(); 
                        if let EvalValue::Variable(nome,value) = v {
                            variable = Some(nome);
                        }
                    } else {
                        return Err("Faltou argumentos para o operador!");
                    }

                    let result = match elem {
                        ExprToken::Attrib => {
                            if let Some(value) = variables.get_mut(variable.unwrap_or("")) {
                                *value = b;

                                *value
                            } else {
                                return Err("Variável não encontrada");
                            }
                        },
                        ExprToken::Mul => a * b,
                        ExprToken::Div => a / b,
                        ExprToken::Rem => a % b,
                        ExprToken::Plus => a + b,
                        ExprToken::Minus => a - b,
                        ExprToken::BitShiftR => ((a as i64) >> (b as i64)) as f64,
                        ExprToken::BitShiftL => ((a as i64) << (b as i64)) as f64,
                        ExprToken::BitAnd => ((a as i64) & (b as i64)) as f64,
                        ExprToken::BitXor => ((a as i64) ^ (b as i64)) as f64,
                        ExprToken::BitOr => ((a as i64) | (b as i64)) as f64,
                        _ => {
                            return Err("Operador desconhecido");
                        }
                    };
                    stack.push(EvalValue::Literal(result));
                } else {
                    let a: f64;
                    if let Some(v) = stack.pop() { a = v.get(); } else {
                        return Err("Faltou argumentos para o operador!");
                    }

                    let result = match elem {
                        ExprToken::UnaryMinus => -a,
                        ExprToken::UnaryPlus => -a,
                        ExprToken::BitNot => (!(a as i64)) as f64,
                        _ => {
                            return Err("Operador unitário desconhecido");
                        }
                    };

                    stack.push(EvalValue::Literal(result));
                }
            }
        }

        if let Some(result) = stack.pop() {
            return Ok(result.get());
        } else {
            return Err("Não tinha nenhum resultado na pilha");
        }
    }

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
    fn parse_expression(expr: &Vec<ExprToken>) -> Vec<ExprToken> {
        let mut output: Vec<ExprToken> = Vec::new();
        let mut stack:  Vec<ExprToken> = Vec::new(); // Pilha com operadores e ( )

        let mut expecting_value = true;
        for mut elem in expr.clone().into_iter() {
            if elem == ExprToken::ParO {
                stack.push(elem);
                expecting_value = true;
            } else if elem == ExprToken::ParC {
                while let Some(stackop) = stack.pop() {
                    if stackop == ExprToken::ParO {
                        break;
                    }
                    output.push(stackop);
                }
                expecting_value = false;
                // stack.pop(); // pop '('
            } else if elem.is_operator() {
                if expecting_value {
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
                expecting_value = true;
            } else if elem.is_literal() {
                output.push(elem);
                expecting_value = false;
            } else if let ExprToken::Word(_) = elem {
                output.push(elem);
                expecting_value = false;
            }
        }

        while let Some(stackop) = stack.pop() {
            output.push(stackop);
        }

        return output;
    }
}

pub fn expression() {
    let mut vars = HashMap::new();
    loop {
        let input = leia(">");

        //println!(" ");

        let result = tokenize(&input);
        if let Err(message) = result {
            println!("Erro: {}", message);
            return;
        }

        let tokens = result.ok().unwrap();
        /*print!("Recebido: ");
        for t in tokens.iter() {
            print!("{} ", t);
        }
        println!();*/

        let parsed = ReversePolish::parse_expression(&tokens);
        /*print!("RPN     : ");
        for t in parsed.iter() {
            print!("{} ", t);
        }
        println!();*/

        //println!(" ");

        let resultado = ReversePolish::eval(&parsed, &mut vars);
        if let Ok(value) = resultado {
            println!("{}", value);
        } else if let Err(message) = resultado {
            println!("Erro ao processar: {}", message);
        }
    }
}
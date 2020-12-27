use std::collections::BTreeMap;
use std::io;
use std::rc::Rc;

// copied from https://stopa.io/post/222

fn main()
{
  use io::Write;

  let env = &mut default_environment();
  loop {
    print!("tack> ");
    io::stdout().flush().expect("failed to flush");
    let exp = read_expression();
    match parse_eval(exp, env) {
      Ok(s) => println!(" => {}", s),
      Err(e) => match e {
        Error::Eval(s) => println!(" ! Error (eval): {}", s),
        Error::Parse(s) => println!(" ! Error (parse): {}", s),
      },
    }
  }
}


#[derive(Debug)]
enum Error
{
  Eval(String),
  Parse(String),
}

impl std::fmt::Display for Error
{
  fn fmt(&self, f: &mut std::fmt::Formatter) ->
    std::fmt::Result
  {
    let cause = match self {
      Error::Eval(msg) => msg,
      Error::Parse(msg) => msg,
    };
    f.write_str(cause)
  }
}

impl std::error::Error for Error
{
  fn description(&self) ->
    &str
  {
    match self {
      Error::Eval(msg) => msg,
      Error::Parse(msg) => msg,
    }
  }
}


#[derive(Clone)]
enum Expression
{
  List(Vec<Expression>),
  Lam(Lambda),
  Fun(fn(&[Expression]) -> Result<Expression, Error>),
  Bool(bool),
  Num(f32),
  Sym(String),
}


#[derive(Clone)]
struct Lambda
{
  params: Rc<Expression>,
  body: Rc<Expression>,
}


impl std::fmt::Display for Expression
{
  fn fmt(&self, f: &mut std::fmt::Formatter) ->
    std::fmt::Result
  {
    let exp = match self {
      Expression::List(l) =>
        ["(".to_string(),
         l.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(","),
         ")".to_string()].join(""),
      Expression::Lam(_) => "Lambda {}".to_string(),
      Expression::Fun(_) => "Function {}".to_string(),
      Expression::Bool(b) => b.to_string(),
      Expression::Num(n) => n.to_string(),
      Expression::Sym(s) => s.clone(),
    };
    f.write_str(&exp)
  }
}


#[derive(Clone)]
struct Environment<'a>
{
  data: BTreeMap<String,Expression>,
  outer: Option<&'a Environment<'a>>,
}


fn tokenise(exp: String) ->
  Vec<String>
{
  exp.replace("(", " ( ").replace(")", " ) ").split_whitespace()
    .map(|t| t.to_string()).collect()
}


fn parse(tokens: &[String]) ->
  Result<(Expression, &[String]), Error>
{
  let (first, rest) = tokens.split_first()
    .ok_or_else(|| Error::Parse("token split_first".to_string()))?;
  match first.as_str() {
    "(" => parse_list(rest),
    ")" => Err(Error::Parse("parens begin".to_string())),
    _ => Ok((parse_atom(first), rest)),
  }
}


fn parse_list(tokens: &[String]) ->
  Result<(Expression, &[String]), Error>
{
  let mut list: Vec<Expression> = vec![];
  let mut s = tokens;
  loop {
    let (first, rest) = s.split_first()
      .ok_or_else(|| Error::Parse("parens end".to_string()))?;
    if first == ")" {
      return Ok((Expression::List(list), rest));
    }
    let (exp, rest) = parse(&s)?;
    list.push(exp);
    s = rest;
  }
}


fn parse_atom(token: &str) ->
  Expression
{
  match token {
    "true" => Expression::Bool(true),
    "false" => Expression::Bool(false),
    _ => match token.parse() {
      Ok(num) => Expression::Num(num),
      Err(_) => Expression::Sym(token.to_string()),
    },
  }
}


fn parse_floats(args: &[Expression]) ->
  Result<Vec<f32>, Error>
{
  args.iter().map(|s| parse_float(s)).collect()
}


fn parse_float(exp: &Expression) ->
  Result<f32, Error>
{
  match exp {
    Expression::Num(num) => Ok(*num),
    _ => Err(Error::Parse("want a number".to_string())),
  }
}


fn parse_args(form: Rc<Expression>) ->
  Result<Vec<String>,Error>
{
  let list = match form.as_ref() {
    Expression::List(s) => Ok(s.clone()),
    _ => Err(Error::Parse("want args to be a list".to_string())),
  }?;
  list.iter().map(|sym| match sym {
    Expression::Sym(sym) => Ok(sym.clone()),
    _ => Err(Error::Parse("want symbols in args".to_string())),
  }).collect()
}


fn eval(exp: &Expression, env: &mut Environment) ->
  Result<Expression,Error>
{
  match exp {
    Expression::List(list) => {
      let first = list.first()
        .ok_or_else(|| Error::Eval("want non-empty list".to_string()))?;
      let args = &list[1..];
      // TODO: ? after this eval?
      match eval_builtin(first, args, env) {
        Some(o) => o,
        None => match eval(first, env)? {
          Expression::Lam(lam) =>
            eval(&lam.body, &mut lambda_environment(lam.params, args, env)?),
          Expression::Fun(fun) =>
            fun(&eval_args(args, env)?),
          _ => Err(Error::Eval("first form must be a function".to_string())),
        },
      }
    },
    Expression::Lam(_) => Err(Error::Eval("unexpected form".to_string())),
    Expression::Fun(_) => Err(Error::Eval("unexpected form".to_string())),
    Expression::Bool(_) => Ok(exp.clone()),
    Expression::Num(_) => Ok(exp.clone()),
    Expression::Sym(sym) => environment_get(sym, env)
      .ok_or_else(|| Error::Eval(["unexpected symbol '", sym, "'"].join("")))
  }
}


fn eval_args(args: &[Expression], env: &mut Environment) ->
  Result<Vec<Expression>,Error>
{
  args.iter().map(|arg| eval(arg, env)).collect()
}


fn eval_builtin(
  exp: &Expression,
  args: &[Expression],
  env: &mut Environment
) -> Option<Result<Expression,Error>>
{
  match exp {
    Expression::Sym(sym) => match sym.as_str() {
      "def" => Some(eval_def(args, env)),
      "if" => Some(eval_if(args, env)),
      "fn" => Some(eval_lambda(args)),
      _ => None,
    },
    _ => None,
  }
}


fn eval_def(args: &[Expression], env: &mut Environment) ->
  Result<Expression,Error>
{
  let first = args.first()
    .ok_or_else(|| Error::Eval("want first form".to_string()))?;
  let first_exp = match first {
    Expression::Sym(sym) => Ok(sym.clone()),
    _ => Err(Error::Eval("want first form to be a symbol".to_string()))
  }?;
  let second = args.get(1)
    .ok_or_else(|| Error::Eval("want second form".to_string()))?;
  if args.len() > 2 {
    return Err(Error::Eval("def only takes two forms".to_string()));
  }
  let second_eval = eval(second, env)?;
  env.data.insert(first_exp, second_eval);
  Ok(first.clone())
}


fn eval_if(args: &[Expression], env: &mut Environment) ->
  Result<Expression,Error>
{
  let cond = args.first()
    .ok_or_else(|| Error::Eval("want conditional".to_string()))?;
  match eval(cond, env)? {
    Expression::Bool(b) => {
      let branch = if b { 1 } else { 2 };
      let consequence = args.get(branch)
        .ok_or_else(|| Error::Eval(["want branch ",
                                    branch.to_string().as_str()].join("")))?;
      eval(consequence, env)
    },
    _ => Err(Error::Eval(["unexpected condition '",
                          cond.to_string().as_str(),
                          "'"].join(""))),
  }
}


fn eval_lambda(args: &[Expression]) ->
  Result<Expression,Error>
{
  let params = args.first()
    .ok_or_else(|| Error::Eval("want args".to_string()))?;
  let body = args.get(1)
    .ok_or_else(|| Error::Eval("want body".to_string()))?;
  if args.len() > 2 {
    return Err(Error::Eval("lambda only takes two forms".to_string()));
  }
  Ok(Expression::Lam(Lambda {
    params: Rc::new(params.clone()),
    body: Rc::new(body.clone()),
  }))
}


fn parse_eval(exp: String, env: &mut Environment) ->
  Result<Expression,Error>
{
  let (p, _) = parse(&tokenise(exp))?;
  let e = eval(&p, env)?;
  Ok(e)
}


fn read_expression() ->
  String
{
  let mut exp = String::new();
  io::stdin().read_line(&mut exp).expect("failed to read in expression");
  exp
}


fn environment_get(
  key: &str,
  env: &Environment
) -> Option<Expression>
{
  match env.data.get(key) {
    Some(exp) => Some(exp.clone()),
    None => {
      match &env.outer {
        Some(outer) => environment_get(key, &outer),
        None => None
      }
    }
  }
}


fn lambda_environment<'a>(
  params: Rc<Expression>,
  args: &[Expression],
  outer: &'a mut Environment
) -> Result<Environment<'a>,Error>
{
  let keys = parse_args(params)?;
  if keys.len() != args.len() {
    return Err(Error::Eval(format!("want {} arguments, but got {}",
                                   keys.len(), args.len())));
  }
  let vals = eval_args(args, outer)?;
  let mut data: BTreeMap<String,Expression> = BTreeMap::new();
  for (k, v) in keys.iter().zip(vals.iter()) {
    data.insert(k.clone(), v.clone());
  }
  Ok(Environment { data, outer: Some(outer), })
}


macro_rules! monotonic
{
  ($check:expr) => {{
    |args: &[Expression]| -> Result<Expression,Error> {
      let floats = parse_floats(args)?;
      let first = floats.first()
        .ok_or_else(|| Error::Parse("want at least one number".to_string()))?;
      let rest = &floats[1..];
      fn f (car: &f32, cdr: &[f32]) -> bool {
        match cdr.first() {
          Some(cadr) => $check(car, cadr) && f(cadr, &cdr[1..]),
          None => true,
        }
      };
      Ok(Expression::Bool(f(first, rest)))
    }
  }};
}


fn default_environment<'a>() ->
  Environment<'a>
{
  let mut data: BTreeMap<String,Expression> = BTreeMap::new();
  data.insert("+".to_string(), Expression::Fun(
    |args: &[Expression]| -> Result<Expression,Error> {
      let sum = parse_floats(args)?.iter().fold(0.0, |sum, a| sum + a);
      Ok(Expression::Num(sum))
    }
  ));
  data.insert("-".to_string(), Expression::Fun(
    |args: &[Expression]| -> Result<Expression,Error> {
      let floats = parse_floats(args)?;
      let first = floats.first()
        .ok_or_else(|| Error::Parse("want at least one number".to_string()))?;
      let sum_rest = floats[1..].iter().fold(0.0, |sum, a| sum + a);
      Ok(Expression::Num(first - sum_rest))
    }
  ));
  data.insert("=".to_string(), Expression::Fun(
    monotonic!(|a, b| a == b)
  ));
  data.insert(">".to_string(), Expression::Fun(
    monotonic!(|a, b| a > b)
  ));
  data.insert(">=".to_string(), Expression::Fun(
    monotonic!(|a, b| a >= b)
  ));
  data.insert("<".to_string(), Expression::Fun(
    monotonic!(|a, b| a < b)
  ));
  data.insert("<=".to_string(), Expression::Fun(
    monotonic!(|a, b| a <= b)
  ));
  Environment { data, outer: None }
}


use super::*;
use crate::lexer::*;
use crate::parser::*;

use std::collections::HashMap;
use std::fmt;
use std::{cell::RefCell, rc::Rc};

pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;

pub enum ControlFlow {
    Error(RuntimeError),
    Return(Value),
    Continue,
    Break,
}

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    locals: HashMap<ExprId, usize>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global_env = Environment::new();

        struct Clock;
        impl fmt::Display for Clock {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "<func clock>")
            }
        }

        impl Callable for Clock {
            fn arity(&self) -> usize {
                0
            }
            fn call(
                &self,
                _interpreter: &mut Interpreter,
                _args: Vec<Value>,
            ) -> RuntimeResult<Value> {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                Ok(Value::Number(now))
            }
        }

        let t = Token::new(
            TokenType::Identifier,
            0,
            String::from("clock"),
            Some(Literal::Nil),
        );
        global_env
            .borrow_mut()
            .define(&t, Value::Callable(Rc::new(Clock)))
            .unwrap();

        Self {
            globals: Rc::clone(&global_env),
            environment: Rc::clone(&global_env),
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> RuntimeResult<()> {
        for stmt in stmts {
            if let Err(ControlFlow::Error(e)) = self.execute(stmt) {
                return Err(e);
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> std::result::Result<(), ControlFlow> {
        match stmt {
            Stmt::Var(stmt) => self.exec_var(stmt)?,
            Stmt::Print(expr) => self.exec_print(expr)?,
            Stmt::Expression(expr) => self.exec_expr(expr)?,
            Stmt::Block(stmts) => self.exec_block(stmts)?,
            Stmt::If(stmt) => self.exec_if(stmt)?,
            Stmt::While(stmt) => self.exec_while(stmt)?,
            Stmt::Function(stmt) => self.exec_func(stmt)?,
            Stmt::Class(stmt) => self.exec_class(stmt)?,
            Stmt::Return(stmt) => self.exec_return(stmt)?,
            Stmt::Break(token) => return Err(ControlFlow::Break),
            Stmt::Continue(token) => return Err(ControlFlow::Continue),
        }
        Ok(())
    }

    fn exec_var(&mut self, stmt: &VarStmt) -> std::result::Result<(), ControlFlow> {
        let value = self.evaluate(&stmt.initializer)?;
        self.environment.borrow_mut().define(&stmt.name, value)?;
        Ok(())
    }

    fn exec_print(&mut self, expr: &Expr) -> std::result::Result<(), ControlFlow> {
        let value = self.evaluate(expr)?;
        println!("{value}");
        Ok(())
    }

    fn exec_expr(&mut self, expr: &Expr) -> std::result::Result<(), ControlFlow> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> std::result::Result<(), ControlFlow> {
        self.execute_block(
            stmts,
            Environment::new_with_env(Rc::clone(&self.environment)),
        )
    }

    fn exec_if(&mut self, stmt: &IfStmt) -> std::result::Result<(), ControlFlow> {
        let value = self.evaluate(&stmt.condition)?;
        if is_truthy(&value) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(stmt) = &stmt.else_branch {
            self.execute(&stmt)?;
        }
        Ok(())
    }

    fn exec_while(&mut self, stmt: &WhileStmt) -> std::result::Result<(), ControlFlow> {
        let mut value = self.evaluate(&stmt.condition)?;
        while is_truthy(&value) {
            match self.execute(&stmt.body) {
                Ok(_) | Err(ControlFlow::Continue) => (),
                Err(ControlFlow::Break) => {
                    break;
                }
                Err(e) => return Err(e),
            }

            if let Some(expr) = &stmt.increment {
                self.evaluate(expr)?;
            }

            value = self.evaluate(&stmt.condition)?;
        }

        Ok(())
    }
    fn exec_func(&mut self, stmt: &FuncStmt) -> std::result::Result<(), ControlFlow> {
        let func = LoxFunction::new(Rc::new(stmt.clone()), Rc::clone(&self.environment), false);
        self.environment
            .borrow_mut()
            .define(&stmt.name, Value::Callable(Rc::new(func)))?;
        Ok(())
    }

    fn exec_class(&mut self, stmt: &ClassStmt) -> std::result::Result<(), ControlFlow> {
        let super_class = if let Some(super_expr) = &stmt.super_class {
            match self.evaluate(super_expr)? {
                Value::Class(class) => Some(class),
                _ => {
                    let token = match &super_expr.kind {
                        ExprKind::Var(var) => &var.token,
                        _ => unreachable!(),
                    };

                    return Err(ControlFlow::Error(RuntimeError {
                        token: Some(token.clone()),
                        message: "super class must be a class.".to_string(),
                    }));
                }
            }
        } else {
            None
        };

        self.environment
            .borrow_mut()
            .define(&stmt.name, Value::Nil)?;

        let prev = self.environment.clone();
        if let Some(super_class) = &super_class {
            let env = Environment::new_with_env(prev.clone());
            env.borrow_mut()
                .define_str("super", Value::Class(super_class.clone()))?;
            self.environment = env;
        }
        let methods = build_methods(&stmt.methods, &self.environment, |m| {
            m.name.lexeme() == "init"
        });
        let class_methods = build_methods(&stmt.class_methods, &self.environment, |_| false);

        let has_super = super_class.is_some();
        let class = LoxClass::new(stmt.name.lexeme(), methods, super_class, class_methods);
        if has_super {
            self.environment = prev;
        }

        self.environment
            .borrow_mut()
            .assign(&stmt.name, Value::Class(Rc::new(class)))?;

        Ok(())
    }

    fn exec_return(&mut self, stmt: &ReturnStmt) -> std::result::Result<(), ControlFlow> {
        let v = self.evaluate(&stmt.value)?;
        return Err(ControlFlow::Return(v));
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    fn evaluate(&mut self, expression: &Expr) -> RuntimeResult<Value> {
        match &expression.kind {
            ExprKind::Literal(l) => literal(l),
            ExprKind::Grouping(g) => self.evaluate(g),
            ExprKind::Get(expr) => self.eval_get(expr),
            ExprKind::Set(expr) => self.eval_set(expr),
            ExprKind::Call(expr) => self.eval_call(expr),
            ExprKind::Unary(expr) => self.eval_unary(expr),
            ExprKind::Binary(expr) => self.eval_binary(expr),
            ExprKind::Logical(expr) => self.eval_logical(expr),
            ExprKind::Function(expr) => self.eval_function(expr),
            ExprKind::Conditional(expr) => self.eval_conditional(expr),
            ExprKind::This(token) => self.lookup_variable(token, expression),
            ExprKind::Var(expr) => self.lookup_variable(&expr.token, expression),
            ExprKind::Super(super_expr) => self.eval_super(expression, super_expr),
            ExprKind::Assignment(ass_expr) => self.eval_assignment(expression, ass_expr),
        }
    }

    fn eval_function(&mut self, expr: &FunctionExpr) -> RuntimeResult<Value> {
        let synthetic_name = Token::new(
            TokenType::Identifier,
            0,
            String::from("anonymous"),
            Some(Literal::Nil),
        );
        let dummy_stmt = FuncStmt {
            name: synthetic_name,
            params: expr.params.clone(),
            body: expr.body.clone(),
            getter: false,
        };

        let closure = Rc::clone(&self.environment);
        let function = LoxFunction::new(Rc::new(dummy_stmt), closure, false);
        Ok(Value::Callable(Rc::new(function)))
    }

    fn eval_super(&mut self, expression: &Expr, super_expr: &SuperExpr) -> RuntimeResult<Value> {
        let distance = *self
            .locals
            .get(&expression.id)
            .ok_or_else(|| RuntimeError::new("super used outside of class"))?;

        let super_class =
            Environment::get_at(self.environment.clone(), distance, &super_expr.keyword)?;

        let instance = Environment::get_at_str(self.environment.clone(), distance - 1, "this")?;

        if let (Value::Class(class), Value::Instance(object)) = (super_class, instance) {
            let method = class
                .find_method(super_expr.method.lexeme())
                .ok_or_else(|| {
                    RuntimeError::new(&format!(
                        "undefined property '{}' on superclass",
                        super_expr.method.lexeme()
                    ))
                })?;

            let bound_method = method.bind(object)?;
            Ok(Value::Callable(Rc::new(bound_method)))
        } else {
            Err(RuntimeError::new(
                "invalid superclass  or execution context",
            ))
        }
    }

    fn eval_unary(&mut self, expr: &UnaryExpr) -> RuntimeResult<Value> {
        let value = self.evaluate(&expr.right)?;
        unary(&expr.operator, value)
    }

    fn eval_binary(&mut self, expr: &BinaryExpr) -> RuntimeResult<Value> {
        let v_left = self.evaluate(&expr.left)?;
        let v_right = self.evaluate(&expr.right)?;
        binary(&expr.operator, v_left, &v_right)
    }

    fn lookup_variable(&mut self, token: &Token, expression: &Expr) -> RuntimeResult<Value> {
        if let Some(&distance) = self.locals.get(&expression.id) {
            Environment::get_at(self.environment.clone(), distance, token)
        } else {
            self.globals.borrow().get(token)
        }
    }

    fn eval_set(&mut self, expr: &SetExpr) -> RuntimeResult<Value> {
        let v = self.evaluate(&expr.object)?;
        if let Value::Instance(instance) = &v {
            let _v = self.evaluate(&expr.value)?;
            instance.set(expr.name.clone(), _v.clone())?;
            return Ok(_v);
        }

        Err(RuntimeError {
            token: Some(expr.name.clone()),
            message: "only instances have fields".to_string(),
        })
    }

    fn eval_get(&mut self, expr: &GetExpr) -> RuntimeResult<Value> {
        let v = self.evaluate(&expr.object)?;

        match &v {
            Value::Instance(instance) => match instance.get(&expr.name)? {
                PropResult::Getter(getter) => {
                    let bound = getter.bind(instance.clone())?;
                    bound.call(self, vec![])
                }
                PropResult::Method(method) => {
                    let bound = method.bind(instance.clone())?;
                    Ok(Value::Callable(Rc::new(bound)))
                }
                PropResult::Value(value) => Ok(value),
            },
            Value::Class(class) => {
                if let Some(class_method) = class.find_class_method(&expr.name.lexeme()) {
                    return Ok(Value::Callable(Rc::new(class_method)));
                }
                Err(RuntimeError {
                    token: Some(expr.name.clone()),
                    message: format!("undefined class method {}", expr.name.lexeme()),
                })
            }
            _ => Err(RuntimeError {
                token: Some(expr.name.clone()),
                message: "only classes and instances have properties".to_string(),
            }),
        }
    }

    fn eval_assignment(
        &mut self,
        expr: &Expr,
        assignment_expr: &AssignmentExpr,
    ) -> RuntimeResult<Value> {
        let v = self.evaluate(&assignment_expr.value)?;
        if let Some(&distance) = self.locals.get(&expr.id) {
            Environment::assign_at(
                self.environment.clone(),
                &assignment_expr.name,
                v.clone(),
                distance,
            )?;
        } else {
            self.globals
                .borrow_mut()
                .assign(&assignment_expr.name, v.clone())?;
        }
        Ok(v)
    }

    fn eval_conditional(&mut self, expr: &ConditionalExpr) -> RuntimeResult<Value> {
        let v_condition = self.evaluate(&expr.condition)?;
        if is_truthy(&v_condition) {
            self.evaluate(&expr.then_branch)
        } else {
            self.evaluate(&expr.else_branch)
        }
    }

    fn eval_logical(&mut self, expr: &LogicalExpr) -> RuntimeResult<Value> {
        let v_left = self.evaluate(&expr.left)?;
        let is_or = expr.operator.token_type() == TokenType::Or;
        if is_truthy(&v_left) == is_or {
            return Ok(v_left);
        }
        self.evaluate(&expr.right)
    }

    fn eval_call(&mut self, expr: &Call) -> RuntimeResult<Value> {
        let callee = self.evaluate(&expr.callee)?;
        let arguments = expr
            .args
            .iter()
            .map(|arg| self.evaluate(arg))
            .collect::<RuntimeResult<_>>()?;

        match callee {
            Value::Callable(func) => {
                if expr.args.len() != func.arity() {
                    return Err(RuntimeError {
                        token: Some(expr.paren.clone()),
                        message: format!(
                            "expected {} arguments, got  {}",
                            func.arity(),
                            expr.args.len()
                        ),
                    });
                }
                func.call(self, arguments)
            }

            /////////////////////////////////////////////////
            // constructor                             //////
            // //////////////////////////////////////////////
            Value::Class(class) => {
                let instance = LoxInstance::new(class.clone());
                if let Some(init) = class.find_method("init") {
                    if expr.args.len() != init.arity() {
                        return Err(RuntimeError {
                            token: Some(expr.paren.clone()),
                            message: format!(
                                "expected {} arguments, got {}",
                                init.arity(),
                                expr.args.len(),
                            ),
                        });
                    }
                    let bound = init.bind(instance.clone())?;
                    bound.call(self, arguments)?;
                }
                Ok(Value::Instance(instance))
            }
            _ => Err(RuntimeError {
                token: Some(expr.paren.clone()),
                message: "can call only callables".to_string(),
            }),
        }
    }

    pub fn execute_block(
        &mut self,
        stmts: &[Stmt],
        env: Rc<RefCell<Environment>>,
    ) -> std::result::Result<(), ControlFlow> {
        let previous = std::mem::replace(&mut self.environment, env);
        for stmt in stmts {
            if let Err(e) = self.execute(stmt) {
                self.environment = previous;
                return Err(e);
            }
        }
        self.environment = previous;
        Ok(())
    }

    pub fn resolve(&mut self, expr: &Expr, i: usize) {
        self.locals.insert(expr.id, i);
    }
}

fn literal(literal: &Literal) -> RuntimeResult<Value> {
    match literal {
        Literal::Number(n) => Ok(Value::Number(*n)),
        Literal::String(s) => Ok(Value::String(s.clone())),
        Literal::True => Ok(Value::Bool(true)),
        Literal::False => Ok(Value::Bool(false)),
        Literal::Nil => Ok(Value::Nil),
    }
}

fn unary(operator: &Token, value: Value) -> RuntimeResult<Value> {
    match operator.token_type() {
        TokenType::Minus => match value {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(RuntimeError {
                token: Some(operator.clone()),
                message: String::from("operand must be a number"),
            }),
        },
        TokenType::Bang => Ok(Value::Bool(!is_truthy(&value))),
        _ => unreachable!(),
    }
}

fn binary(operator: &Token, left: Value, right: &Value) -> RuntimeResult<Value> {
    let err = |msg: &str| {
        Err(RuntimeError {
            token: Some(operator.clone()),
            message: String::from(msg),
        })
    };

    match operator.token_type() {
        TokenType::Plus => left.add(right),
        TokenType::Minus => left.sub(right),
        TokenType::Star => left.mul(right),
        TokenType::Slash => left.divide(right),
        TokenType::EqualEqual => left.equal(right),
        TokenType::BangEqual => left.not_equal(right),
        TokenType::Lesser => left.lesser(right),
        TokenType::Greater => left.greater(right),
        TokenType::LesserEqual => left.lesser_equal(right),
        TokenType::GreaterEqual => left.greater_equal(right),
        _ => err("unknown operator"),
    }
    .map_err(|mut e| {
        e.token = Some(operator.clone());
        e
    })
}

fn build_methods(
    funcs: &[FuncStmt],
    env: &Rc<RefCell<Environment>>,
    is_initializer: impl Fn(&FuncStmt) -> bool,
) -> HashMap<String, LoxFunction> {
    funcs
        .iter()
        .map(|f| {
            let func = LoxFunction::new(Rc::new(f.clone()), env.clone(), is_initializer(f));
            (f.name.lexeme().to_string(), func)
        })
        .collect()
}

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::Number(n) => *n != 0.0,
        _ => false,
    }
}

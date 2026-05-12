pub mod symbol;
pub mod ty;

use std::collections::HashMap;
use tinycolor::Colorize;
use strsim::jaro_winkler;
use symbol::*;
use ty::*;
use crate::operator::Operator;
use crate::parser::{ast::*, ty::*};
use crate::diagnostic::Diagnostic;
use crate::span::Span;

const CANDIDATE_SCORE_THRESHOLD: f64 = 0.70;

pub struct SemChecker<'a> {
    pub rodeo: &'a mut lasso::Rodeo,
    pub path: &'a str,
    pub no_color: bool,
    pub symbols: Vec<SymbolMap>,
    pub type_map: HashMap<NodeId, TypeId>,
    pub functions: HashMap<FuncId, FunctionData>,
    pub function_decls: HashMap<NodeId, FuncId>,
    pub types: HashMap<TypeId, Type>,
    pub type_registry: HashMap<lasso::Spur, TypeId>,
    pub next_type_id: usize,
    pub next_func_id: usize,
    pub unit_id: TypeId,
    pub unknown_id: TypeId,
    pub current_function: Option<FuncId>,
}

impl<'a> SemChecker<'a> {
    pub fn new(rodeo: &'a mut lasso::Rodeo, path: &'a str, no_color: bool) -> Self {
        let i8_spur = rodeo.get_or_intern("i8");
        let i16_spur = rodeo.get_or_intern("i16");
        let i32_spur = rodeo.get_or_intern("i32");
        let i64_spur = rodeo.get_or_intern("i64");
        let isz_spur = rodeo.get_or_intern("isz");
        let u8_spur = rodeo.get_or_intern("u8");
        let u16_spur = rodeo.get_or_intern("u16");
        let u32_spur = rodeo.get_or_intern("u32");
        let u64_spur = rodeo.get_or_intern("u64");
        let usz_spur = rodeo.get_or_intern("usz");
        let bool_spur = rodeo.get_or_intern("bool");

        let mut schecker = Self {
            rodeo, path, no_color,
            symbols: vec![SymbolMap::new(MapScope::Root)],
            type_map: HashMap::new(),
            functions: HashMap::new(),
            function_decls: HashMap::new(),
            types: HashMap::new(),
            type_registry: HashMap::new(),
            next_type_id: 0,
            next_func_id: 0,
            unit_id: TypeId(0),
            unknown_id: TypeId(0),
            current_function: None
        };

        schecker.unknown_id = schecker.create_type(Type::Unknown);
        let i8_id = schecker.create_type(Type::I8);
        let i16_id = schecker.create_type(Type::I16);
        let i32_id = schecker.create_type(Type::I32);
        let i64_id = schecker.create_type(Type::I64);
        let isz_id = schecker.create_type(Type::Isz);
        let u8_id = schecker.create_type(Type::U8);
        let u16_id = schecker.create_type(Type::U16);
        let u32_id = schecker.create_type(Type::U32);
        let u64_id = schecker.create_type(Type::U64);
        let usz_id = schecker.create_type(Type::Usz);
        schecker.unit_id = schecker.create_type(Type::Unit);
        let bool_id = schecker.create_type(Type::Bool);

        schecker.type_registry.insert(i8_spur, i8_id);
        schecker.type_registry.insert(i16_spur, i16_id);
        schecker.type_registry.insert(i32_spur, i32_id);
        schecker.type_registry.insert(i64_spur, i64_id);
        schecker.type_registry.insert(isz_spur, isz_id);
        schecker.type_registry.insert(u8_spur, u8_id);
        schecker.type_registry.insert(u16_spur,u16_id);
        schecker.type_registry.insert(u32_spur, u32_id);
        schecker.type_registry.insert(u64_spur, u64_id);
        schecker.type_registry.insert(usz_spur, usz_id);
        schecker.type_registry.insert(bool_spur, bool_id);

        schecker
    }

    fn create_type(&mut self, ty: Type) -> TypeId {
        let id = TypeId(self.next_type_id);
        self.next_type_id += 1;
        self.types.insert(id, ty);
        id
    }

    fn get_ident_type(&self, name: &lasso::Spur) -> Result<&TypeId, Option<lasso::Spur>> {
        let mut candidate = None;
        let mut candidate_score = 0.0;

        for map in self.symbols.iter().rev() {
            match map.scope {
                MapScope::Function(f) => if let Some(fid) = self.current_function {
                    if fid != f { continue }
                }, 
                _ => (),
            }
            for (n, ty) in map.iter_types() {
                if *name == *n {
                    return Ok(ty);
                } else {
                    let resolved_name = self.rodeo.resolve(name);
                    let resolved_candidate = self.rodeo.resolve(n);
                    let score = jaro_winkler(resolved_name, resolved_candidate);
                    if score >= CANDIDATE_SCORE_THRESHOLD && score > candidate_score {
                        candidate = Some(*n);
                        candidate_score = score;
                    }
                }
            }
        }
        Err(candidate)
    }

    fn resolve_type(&self, ty: &ParseType) -> Result<&TypeId, Diagnostic> {
        match &ty.kind {
            ParseTypeKind::Identifier(s) => {
                let mut candidate = None;
                let mut candidate_score = 0.0;
                for (n, ty) in self.type_registry.iter() {
                    if *s == *n {
                        return Ok(ty);
                    } else {
                        let resolved_name = self.rodeo.resolve(s);
                        let resolved_candidate = self.rodeo.resolve(n);
                        let score = jaro_winkler(resolved_name, resolved_candidate);
                        if score >= CANDIDATE_SCORE_THRESHOLD && score > candidate_score {
                            candidate = Some(*n);
                            candidate_score = score;
                        }
                    }
                }
                Err(Diagnostic {
                    path: self.path.to_string(),
                    msg: format!(
                        "`{}` not in scope",
                        self.rodeo.resolve(s)
                    ),
                    span: ty.span,
                    no_color: self.no_color,
                    secondaries: vec![
                        (candidate.as_ref().map(|sp| format!(
                            "{}: did you mean `{}`?",
                            if !self.no_color {
                                "help".blue().bold().to_string()
                            } else {
                                "help".to_string()
                            },
                            self.rodeo.resolve(sp)
                        )), None)
                    ],
                })
            }
        }
    }

    pub fn check(&mut self, ast: &Ast) -> Result<(), Vec<Diagnostic>> {
        let mut errors = vec![];

        for node in ast.0.iter() {
            if let Err(err) = self.check_root_level_item(node) {
                errors.extend(err);
            }
        }

        let main_spur = self.rodeo.get_or_intern("main");
        if self.symbols[0].get_type(&main_spur).is_none() {
            errors.push(Diagnostic {
                path: self.path.to_string(),
                msg: "expected `main` function entry point".to_string(),
                span: Span { start: 0, end: 1 },
                no_color: self.no_color,
                secondaries: vec![],
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn check_root_level_item(&mut self, node: &Expr) -> Result<(), Vec<Diagnostic>> {
        match &node.kind {
            ExprKind::FunctionDecl { .. } | ExprKind::FunctionDef { .. } => self.check_node(node),
            ExprKind::Semi(stmt) => self.check_root_level_item(stmt),
            _ => Err(vec![Diagnostic {
                path: self.path.to_string(),
                msg: format!("expected root-level item"),
                span: node.span,
                no_color: self.no_color,
                secondaries: vec![],
            }])
        }
    }

    pub fn check_node(&mut self, node: &Expr) -> Result<(), Vec<Diagnostic>> {
        let result;

        match &node.kind {
            ExprKind::Int(_) => result = self.create_type(Type::AmbiguousInt), // ambiguous types are independent
            ExprKind::Float(_) => result = self.create_type(Type::AmbiguousFloat),
            ExprKind::String(_) => todo!("strings"),
            ExprKind::Identifier(s) => result = *self.get_ident_type(s)
                .map_err(|candidate| {
                    Diagnostic {
                        path: self.path.to_string(),
                        msg: format!(
                            "`{}` not in scope",
                            self.rodeo.resolve(s),
                        ),
                        span: node.span,
                        no_color: self.no_color,
                        secondaries: vec![
                            (candidate.as_ref().map(|sp| format!(
                                "{}: did you mean `{}`?",
                                if !self.no_color {
                                    "help".blue().bold().to_string()
                                } else {
                                    "help".to_string()
                                },
                                self.rodeo.resolve(sp)
                            )), None)
                        ],
                    }
                }).map_err(|err| vec![err])?,
            ExprKind::Semi(stmt) => {
                self.check_node(stmt)?;
                result = self.unit_id;
            },
            ExprKind::Block(stmts) => {
                let mut final_ty = None;
                self.symbols.push(SymbolMap::new(MapScope::Function(self.current_function.unwrap())));
                for stmt in stmts {
                    self.check_node(stmt)?;
                    final_ty = Some(self.type_map[&stmt.id]);
                }
                self.symbols.pop();
                result = final_ty.unwrap_or(self.unit_id);
            },
            ExprKind::BinaryOp { lhs, rhs, op } => {
                if *op == Operator::Assign {
                    if let ExprKind::Identifier(s) = &lhs.kind {
                        let ty = *self.get_ident_type(s)
                            .map_err(|candidate| {
                                Diagnostic {
                                    path: self.path.to_string(),
                                    msg: format!(
                                        "`{}` not in scope",
                                        self.rodeo.resolve(s),
                                    ),
                                    span: node.span,
                                    no_color: self.no_color,
                                    secondaries: vec![
                                        (candidate.as_ref().map(|sp| format!(
                                            "{}: did you mean `{}`?",
                                            if !self.no_color {
                                                "help".blue().bold().to_string()
                                            } else {
                                                "help".to_string()
                                            },
                                            self.rodeo.resolve(sp)
                                        )), None)
                                    ],
                                }
                            }).map_err(|err| vec![err])?;
                        self.check_node(rhs)?;

                        let rhs_id = &self.type_map[&rhs.id];
                        if self.types[&ty].is_coerceable(&self.types[rhs_id]) {
                            self.types.insert(ty, self.types[rhs_id].clone());
                        } else if self.types[rhs_id].is_coerceable(&self.types[&ty]) {
                            self.types.insert(*rhs_id, self.types[&ty].clone());
                        } else if self.types[&ty] != self.types[rhs_id] {
                            return Err(vec![Diagnostic {
                                path: self.path.to_string(),
                                msg: format!(
                                    "`{}` has type `{}` but found `{}`",
                                    self.rodeo.resolve(s),
                                    self.types[&ty].debug(&self.types),
                                    self.types[rhs_id].debug(&self.types),
                                ),
                                span: lhs.span,
                                no_color: self.no_color,
                                secondaries: vec![]
                            }]);
                        }
                        self.type_map.insert(node.id, ty);
                        return Ok(());
                    } else {
                        return Err(vec![Diagnostic {
                            path: self.path.to_string(),
                            msg: "invalid mutation target".to_string(),
                            span: lhs.span,
                            no_color: self.no_color,
                            secondaries: vec![]
                        }]);
                    }
                }
                let mut errors = vec![];
                if let Err(err) = self.check_node(lhs) {
                    errors.extend(err);
                }
                if let Err(err) = self.check_node(rhs) {
                    errors.extend(err);
                }
                if !errors.is_empty() { return Err(errors) }

                let lhs_id = &self.type_map[&lhs.id];
                let rhs_id = &self.type_map[&rhs.id];

                if let Some(ty) = op.infix_output_type(lhs_id, rhs_id, &mut self.types) {
                    result = ty;
                } else {
                    return Err(vec![Diagnostic {
                        path: self.path.to_string(),
                        msg: format!(
                            "cannot do `{op}` operation on types `{}` and `{}`",
                            self.types[lhs_id].debug(&self.types),
                            self.types[rhs_id].debug(&self.types)
                        ),
                        span: node.span,
                        no_color: self.no_color,
                        secondaries: vec![],
                    }]);
                }
            },
            ExprKind::UnaryOp { operand, op } => {
                self.check_node(operand)?;
                let operand_id = &self.type_map[&operand.id];
                if let Some(ty) = op.prefix_output_type(operand_id, &self.types) {
                    result = ty;
                } else {
                    return Err(vec![Diagnostic {
                        path: self.path.to_string(),
                        msg: format!(
                            "cannot do `{op}` operation on type `{}`",
                            self.types[operand_id].debug(&self.types)
                        ),
                        span: node.span,
                        no_color: self.no_color,
                        secondaries: vec![],
                    }]);
                }
            },
            ExprKind::Let { name, ty, init } => {
                let ty = if let Some(ty) = ty {
                    *self.resolve_type(ty).map_err(|err| vec![err])?
                } else { self.unknown_id };
                let init_ty = {
                    self.check_node(init)?;
                    self.type_map[&init.id]
                };
                let final_ty = if self.types[&init_ty].is_coerceable(&self.types[&ty]) {
                    self.type_map.insert(init.id, ty);
                    ty
                } else { init_ty };
                self.symbols.last_mut().unwrap()
                    .define_symbol(*name,
                        false,
                        final_ty,
                    );
                result = final_ty;
            },
            ExprKind::Var { name, ty, init } => {
                let ty = if let Some(ty) = ty {
                    *self.resolve_type(ty).map_err(|err| vec![err])?
                } else { self.unknown_id };
                let init_ty = {
                    self.check_node(init)?;
                    self.type_map[&init.id]
                };
                let final_ty = if self.types[&init_ty].is_coerceable(&self.types[&ty]) {
                    self.type_map.insert(init.id, ty);
                    ty
                } else { init_ty };
                self.symbols.last_mut().unwrap()
                    .define_symbol(*name,
                        true,
                        final_ty,
                    );
                result = final_ty;
            },
            ExprKind::If { condition, then_body, else_body } => {
                let mut errors = vec![];
                if let Err(err) = self.check_node(condition) { errors.extend(err) }
                self.symbols.push(SymbolMap::new(MapScope::Function(self.current_function.unwrap())));
                if let Err(err) = self.check_node(then_body) { errors.extend(err) }
                self.symbols.pop();
                if let Some(expr) = else_body {
                    self.symbols.push(SymbolMap::new(MapScope::Function(self.current_function.unwrap())));
                    if let Err(err) = self.check_node(expr) { errors.extend(err) }
                    self.symbols.pop();
                }
                if !errors.is_empty() { return Err(errors) }
                let condition_ty = &self.types[&self.type_map[&condition.id]];
                if *condition_ty != Type::Bool {
                    errors.push(Diagnostic {
                        path: self.path.to_string(),
                        msg: format!("expected `bool` condition, found `{}`", condition_ty.debug(&self.types)),
                        span: condition.span,
                        no_color: self.no_color,
                        secondaries: vec![],
                    });
                }
                let then_ty = &self.types[&self.type_map[&then_body.id]];
                if let Some(else_body) = else_body {
                    let else_ty = &self.types[&self.type_map[&else_body.id]];
                    if then_ty.is_coerceable(else_ty) {
                        self.type_map.insert(then_body.id, self.type_map[&else_body.id]);
                    } else if else_ty.is_coerceable(then_ty) {
                        self.type_map.insert(else_body.id, self.type_map[&then_body.id]);
                    } else {
                        errors.push(Diagnostic {
                            path: self.path.to_string(),
                            msg: format!("expected `else` clause with type `{}`", then_ty.debug(&self.types)),
                            span: condition.span,
                            no_color: self.no_color,
                            secondaries: vec![],
                        });
                    }
                } else if *then_ty != Type::Unit {
                    errors.push(Diagnostic {
                        path: self.path.to_string(),
                        msg: format!("expected `else` clause with type `{}`", then_ty.debug(&self.types)),
                        span: condition.span,
                        no_color: self.no_color,
                        secondaries: vec![],
                    });
                }

                result = self.type_map[&then_body.id];
            },
            ExprKind::While { condition, body, cont_expr } => {
                let mut errors = vec![];
                if let Err(err) = self.check_node(condition) { errors.extend(err) }
                self.symbols.push(SymbolMap::new(MapScope::Function(self.current_function.unwrap())));
                if let Err(err) = self.check_node(body) { errors.extend(err) }
                self.symbols.pop();
                if let Some(expr) = cont_expr {
                    self.symbols.push(SymbolMap::new(MapScope::Function(self.current_function.unwrap())));
                    if let Err(err) = self.check_node(expr) { errors.extend(err) }
                    self.symbols.pop();
                }
                if !errors.is_empty() { return Err(errors) }
                let condition_ty = &self.types[&self.type_map[&condition.id]];
                if *condition_ty != Type::Bool {
                    errors.push(Diagnostic {
                        path: self.path.to_string(),
                        msg: format!("expected `bool` condition, found `{}`", condition_ty.debug(&self.types)),
                        span: condition.span,
                        no_color: self.no_color,
                        secondaries: vec![],
                    });
                }
                
                result = self.type_map[&body.id];
            },
            ExprKind::FunctionDef { name, params, return_ty, body, unwrap } => {
                let mut param_tys = vec![];
                let mut errors = vec![];
                for param in params {
                    match self.resolve_type(&param.ty) {
                        Ok(ty) => param_tys.push(*ty),
                        Err(err) => errors.push(err),
                    }
                }
                let ret_ty = match return_ty.as_ref().map(|ty| self.resolve_type(&ty))
                    .unwrap_or(Ok(&self.unit_id))
                {
                    Ok(ty) => *ty,
                    Err(err) => {
                        errors.push(err);
                        return Err(errors);
                    },
                };
                if !errors.is_empty() { return Err(errors) }

                let test_fid = FuncId(self.next_func_id);
                let ty = Type::Function(*unwrap, param_tys.clone(), ret_ty, test_fid);
                let mut already_declared = false;
                let (func_type_id, fid) = if let Some(n) = self.symbols.last().unwrap().get_type(name) {
                    if let Type::Function(real_unwrap, real_param_tys, real_ret_ty, real_fid) = &self.types[n] {
                        let mut params_equal = false;
                        if param_tys.len() == real_param_tys.len() {
                            for (pty, rty) in param_tys.iter().zip(real_param_tys) {
                                params_equal |= self.types[pty] == self.types[rty];
                            }
                        }
                        if (*real_unwrap == *unwrap) && params_equal && (self.types[&ret_ty] == self.types[real_ret_ty]) {
                            already_declared = true;
                            (*n, *real_fid)
                        } else {
                            return Err(vec![Diagnostic {
                                path: self.path.to_string(),
                                msg: format!("conflicting function declaration and definition"),
                                span: node.span,
                                no_color: self.no_color,
                                secondaries: vec![],
                            }]);
                        }
                    } else {
                        self.next_func_id += 1;
                        (self.create_type(ty), test_fid)
                    }
                } else {
                        self.next_func_id += 1;
                    (self.create_type(ty), test_fid)
                };

                self.symbols.last_mut().unwrap()
                    .define_symbol(*name, false, func_type_id);
                self.function_decls.insert(node.id, fid);
                if !already_declared {
                    self.functions.insert(fid, FunctionData { param_tys: param_tys.clone(), ret_ty, fty: func_type_id });
                }

                let mut smap = SymbolMap::new(MapScope::Function(fid));
                let old_function = self.current_function;
                self.current_function = Some(fid);
                for (param, resolved_ty) in params.iter().zip(param_tys) {
                    smap.define_symbol(param.name.0, param.mutability, resolved_ty);
                }
                self.symbols.push(smap);
                self.check_node(body)?;
                self.symbols.pop();

                let body_ty = &self.types[&self.type_map[&body.id]];
                if body_ty.is_coerceable(&self.types[&ret_ty])
                    || (*body_ty == self.types[&ret_ty] && !body_ty.is_ambiguous())
                {
                    self.type_map.insert(body.id, ret_ty);
                } else {
                    return Err(vec![Diagnostic {
                        path: self.path.to_string(),
                        msg: format!(
                            "function declared with return type `{}` but returns `{}`",
                            self.types[&ret_ty].debug(&self.types),
                            body_ty.debug(&self.types)
                        ),
                        span: body.span,
                        no_color: self.no_color,
                        secondaries: vec![],
                    }]);
                }

                self.current_function = old_function;
                result = self.unit_id;
            },
            ExprKind::FunctionDecl { name, params, return_ty, unwrap } => {
                let mut param_tys = vec![];
                let mut errors = vec![];
                for param in params {
                    match self.resolve_type(&param.ty) {
                        Ok(ty) => param_tys.push(*ty),
                        Err(err) => errors.push(err),
                    }
                }
                let ret_ty = match return_ty.as_ref().map(|ty| self.resolve_type(&ty))
                    .unwrap_or(Ok(&self.unit_id))
                {
                    Ok(ty) => *ty,
                    Err(err) => {
                        errors.push(err);
                        return Err(errors);
                    },
                };
                if !errors.is_empty() { return Err(errors) }

                let fid = FuncId(self.next_func_id);
                self.next_func_id += 1;
                let func_type_id = self.create_type(Type::Function(*unwrap, param_tys.clone(), ret_ty, fid));
                self.symbols.last_mut().unwrap().define_symbol(*name, false, func_type_id);
                self.function_decls.insert(node.id, fid);
                self.functions.insert(fid, FunctionData { param_tys, ret_ty, fty: func_type_id });

                result = self.unit_id;
            },
            ExprKind::FunctionCall { callee, args } => {
                self.check_node(callee)?;
                let func_ty = self.types[&self.type_map[&callee.id]].clone();
                if let Type::Function(_, params, ret, _) = func_ty {
                    let args_len = args.len();
                    let params_len = params.len();
                    if args_len != params_len {
                        return Err(vec![Diagnostic {
                            path: self.path.to_string(),
                            msg: format!("expected {params_len} arguments, found {args_len} arguments"),
                            span: node.span,
                            no_color: self.no_color,
                            secondaries: vec![],
                        }]);
                    }
                    for (idx, (arg, param)) in args.iter().zip(params).enumerate() {
                        self.check_node(arg)?;
                        let arg_ty = &self.types[&self.type_map[&arg.id]];
                        let param_ty = &self.types[&param];
                        if !(arg_ty.is_coerceable(param_ty) || *arg_ty == *param_ty) {
                            return Err(vec![Diagnostic {
                                path: self.path.to_string(),
                                msg: format!(
                                    "argument #{} expects `{}` but found `{}",
                                    idx + 1,
                                    param_ty.debug(&self.types),
                                    arg_ty.debug(&self.types)
                                ),
                                span: arg.span,
                                no_color: self.no_color,
                                secondaries: vec![],
                            }]);
                        } else if arg_ty.is_ambiguous() {
                            self.type_map.insert(arg.id, param);
                        }
                    }
                    result = ret;
                } else {
                    return Err(vec![Diagnostic {
                        path: self.path.to_string(),
                        msg: format!("cannot call type `{}`", func_ty.debug(&self.types)),
                        span: callee.span,
                        no_color: self.no_color,
                        secondaries: vec![],
                    }]);
                }
            },
        }

        self.type_map.insert(node.id, result);
        Ok(())
    }
}

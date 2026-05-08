pub mod symbol;
pub mod ty;

use std::collections::HashMap;
use strsim::jaro_winkler;
use symbol::SymbolMap;
use ty::{Type, TypeId};
use crate::parser::{ast::*, ty::*};
use crate::diagnostic::Diagnostic;

const CANDIDATE_SCORE_THRESHOLD: f64 = 0.70;

pub struct SemChecker<'a> {
    pub rodeo: &'a lasso::Rodeo,
    pub path: &'a str,
    pub no_color: bool,
    pub symbols: Vec<SymbolMap>,
    pub type_map: HashMap<NodeId, TypeId>,
    pub types: HashMap<TypeId, Type>,
    pub type_registry: HashMap<lasso::Spur, TypeId>,
    pub next_type_id: usize,
    pub unit_id: TypeId,
    pub unknown_id: TypeId,
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

        let mut schecker = Self {
            rodeo, path, no_color,
            symbols: vec![SymbolMap::new()],
            type_map: HashMap::new(),
            types: HashMap::new(),
            type_registry: HashMap::new(),
            next_type_id: 0,
            unit_id: TypeId(0),
            unknown_id: TypeId(0)
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
                        "`{}` not in scope{}",
                        self.rodeo.resolve(s),
                        candidate.as_ref().map(|sp| format!(
                            " (did you mean `{}`?)",
                            self.rodeo.resolve(sp)
                        )).unwrap_or("".to_string())
                    ),
                    span: ty.span,
                    no_color: self.no_color
                })
            }
        }
    }

    pub fn check(&mut self, ast: &Ast) -> Result<(), Vec<Diagnostic>> {
        let mut errors = vec![];

        for node in ast.0.iter() {
            if let Err(err) = self.check_node(node) {
                errors.extend(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
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
                            "`{}` not in scope{}",
                            self.rodeo.resolve(s),
                            candidate.as_ref().map(|sp| format!(
                                " (did you mean `{}`?)",
                                self.rodeo.resolve(sp)
                            )).unwrap_or("".to_string())
                        ),
                        span: node.span,
                        no_color: self.no_color,
                    }
                }).map_err(|err| vec![err])?,
            ExprKind::Semi(stmt) => {
                self.check_node(stmt)?;
                result = self.unit_id;
            },
            ExprKind::Block(stmts) => {
                let mut final_ty = None;
                for stmt in stmts {
                    self.check_node(stmt)?;
                    final_ty = Some(self.type_map[&stmt.id]);
                }
                result = final_ty.unwrap_or(self.unit_id);
            },
            ExprKind::BinaryOp { lhs, rhs, op } => {
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
                        no_color: self.no_color
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
                        no_color: self.no_color
                    }]);
                }
            },
            ExprKind::Let { name, ty, init } => {
                let ty = if let Some(ty) = ty {
                    *self.resolve_type(ty).map_err(|err| vec![err])?
                } else { self.unknown_id };
                let init_ty = if let Some(init) = init {
                    self.check_node(init)?;
                    self.type_map[&init.id]
                } else { self.unknown_id };
                let final_ty = if self.types[&init_ty].is_coerceable(&self.types[&ty]) {
                    if let Some(init) = init {
                        self.type_map.insert(init.id, ty);
                    }
                    ty
                } else { init_ty };
                self.symbols.last_mut().unwrap()
                    .define_symbol(*name, false, final_ty);
                result = final_ty;
            },
            ExprKind::Var { name, ty, init } => {
                let ty = if let Some(ty) = ty {
                    *self.resolve_type(ty).map_err(|err| vec![err])?
                } else { self.unknown_id };
                let init_ty = if let Some(init) = init {
                    self.check_node(init)?;
                    self.type_map[&init.id]
                } else { self.unknown_id };
                let final_ty = if self.types[&init_ty].is_coerceable(&self.types[&ty]) {
                    if let Some(init) = init {
                        self.type_map.insert(init.id, ty);
                    }
                    ty
                } else { init_ty };
                self.symbols.last_mut().unwrap()
                    .define_symbol(*name, true, final_ty);
                result = final_ty;
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

                let ty = Type::Function(*unwrap, param_tys.clone(), ret_ty);
                let func_type_id = if let Some(n) = self.symbols.last().unwrap().get_type(name) {
                    if self.types[n] == ty {
                        *n
                    } else {
                        return Err(vec![Diagnostic {
                            path: self.path.to_string(),
                            msg: format!("conflicting function declaration and definition"),
                            span: node.span,
                            no_color: self.no_color
                        }]);
                    }
                } else {
                    self.create_type(ty)
                };

                self.symbols.last_mut().unwrap()
                    .define_symbol(*name, false, func_type_id);

                let mut smap = SymbolMap::new();
                for (param, resolved_ty) in params.iter().zip(param_tys) {
                    smap.define_symbol(param.name.0, param.mutability, resolved_ty);
                }
                self.symbols.push(smap);
                self.check_node(body)?;
                self.symbols.pop();

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

                let func_type_id = self.create_type(Type::Function(*unwrap, param_tys.clone(), ret_ty));
                self.symbols.last_mut().unwrap()
                    .define_symbol(*name, false, func_type_id);

                result = self.unit_id;
            },
            ExprKind::FunctionCall { callee, args } => {
                self.check_node(callee)?;
                if let Type::Function(_, params, ret) = self.types[&self.type_map[&callee.id]].clone() {
                    let args_len = args.len();
                    let params_len = params.len();
                    if args_len != params_len {
                        return Err(vec![Diagnostic {
                            path: self.path.to_string(),
                            msg: format!("expected {params_len} arguments, found {args_len} arguments"),
                            span: node.span,
                            no_color: self.no_color
                        }]);
                    }
                    for (idx, (arg, param)) in args.iter().zip(params).enumerate() {
                        self.check_node(arg)?;
                        let arg_ty = &self.types[&self.type_map[&arg.id]];
                        let param_ty = &self.types[&param];
                        if !arg_ty.is_coerceable(param_ty) {
                            return Err(vec![Diagnostic {
                                path: self.path.to_string(),
                                msg: format!(
                                    "argument #{} expects `{}` but found `{}",
                                    idx + 1,
                                    param_ty.debug(&self.types),
                                    arg_ty.debug(&self.types)
                                ),
                                span: arg.span,
                                no_color: self.no_color
                            }]);
                        } else if arg_ty.is_ambiguous() {
                            self.type_map.insert(arg.id, param);
                        }
                    }
                    result = ret;
                } else {
                    return Err(vec![Diagnostic {
                        path: self.path.to_string(),
                        msg: format!("cannot call a non-function"),
                        span: callee.span,
                        no_color: self.no_color
                    }]);
                }
            },
        }

        self.type_map.insert(node.id, result);
        Ok(())
    }
}

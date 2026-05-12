pub mod error;
pub mod symbol;

use error::*;
use symbol::*;
use target_lexicon::Triple;
use std::collections::HashMap;
use cranelift::prelude::*;
use cranelift_codegen::ir::{BlockArg, Function};
use cranelift_module::{Linkage, Module, FuncId as IrFuncId};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use crate::operator::Operator;
use crate::diagnostic::Diagnostic;
use crate::parser::ast::*;
use crate::sema::symbol::{FuncId, FunctionData};
use crate::sema::ty::{Type as SemType, TypeId};

pub struct IRGenerator<'a> {
    path: &'a str,
    rodeo: &'a lasso::Rodeo,
    module: ObjectModule,
    symbols: Vec<HashMap<lasso::Spur, Symbol>>,
    type_map: &'a HashMap<NodeId, TypeId>,
    functions: &'a HashMap<FuncId, FunctionData>,
    function_decls: &'a HashMap<NodeId, FuncId>,
    types: &'a HashMap<TypeId, SemType>,
    ir_functions: HashMap<FuncId, (IrFuncId, Signature)>,
    no_color: bool,
}

impl<'a> IRGenerator<'a> {
    pub fn new(
        name: &str,
        path: &'a str,
        rodeo: &'a lasso::Rodeo,
        type_map: &'a HashMap<NodeId, TypeId>,
        functions: &'a HashMap<FuncId, FunctionData>,
        function_decls: &'a HashMap<NodeId, FuncId>,
        types: &'a HashMap<TypeId, SemType>,
        no_color: bool,
    ) -> Result<Self, IrGenError> {
        let isa_builder = cranelift_native::builder()
            .map_err(|err| IrGenError::BackendError(err.into(), no_color))?;
        let isa = isa_builder.finish(settings::Flags::new(settings::builder()))
            .map_err(|err| IrGenError::BackendError(err.into(), no_color))?;

        let obj_builder = ObjectBuilder::new(isa, name, cranelift_module::default_libcall_names())
            .map_err(|err| IrGenError::BackendError(err.into(), no_color))?;
        let module = ObjectModule::new(obj_builder);

        Ok(Self {
            module, rodeo, path,
            symbols: vec![HashMap::new()],
            type_map, functions,
            function_decls, types,
            ir_functions: HashMap::new(),
            no_color
        })
    }

    pub fn target(&self) -> &Triple {
        self.module.isa().triple()
    }

    fn find_ident(&self, name: &lasso::Spur) -> Option<&Symbol> {
        for s in self.symbols.iter().rev() {
            if let Some(symbol) = s.get(name) {
                return Some(symbol);
            }
        }

        None
    }

    pub fn generate(mut self, ast: &Ast) -> Result<ObjectProduct, IrGenError> {
        for node in ast.0.iter() {
            self.walk_root_level_item(node)?;
        }

        Ok(self.module.finish())
    }

    fn walk_root_level_item(&mut self, node: &Expr) -> Result<(), IrGenError> {
        match &node.kind {
            ExprKind::FunctionDecl { name, params, return_ty, .. } => {
                let func_id = &self.function_decls[&node.id];
                let data = &self.functions[func_id];
                let mut signature = self.module.make_signature();
                let target = self.module.isa().triple();
                for (ty, param) in data.param_tys.iter().zip(params) {
                    signature.params.push(AbiParam::new(
                        self.types[ty].into_clif_type(target, self.path, param.ty.span, self.no_color)
                            .map_err(IrGenError::Diagnostic)?
                    ));
                }
                let data_ret_ty = &self.types[&data.ret_ty];
                if *data_ret_ty != SemType::Unit {
                    signature.returns.push(AbiParam::new(
                        data_ret_ty.into_clif_type(target, self.path, return_ty.as_ref().unwrap().span, self.no_color)
                            .map_err(IrGenError::Diagnostic)?
                    ));
                } else {
                    signature.returns.push(AbiParam::new(types::I8));
                }

                let string_name = if self.symbols.len() == 1 {
                    self.rodeo.resolve(name).to_string()
                } else {
                    format!("{}_{}", self.rodeo.resolve(name), self.symbols.len())
                };
                let ir_func_id = self.module.declare_function(&string_name, Linkage::Export, &signature)
                    .map_err(|err| IrGenError::BackendError(err.into(), self.no_color))?;
                self.ir_functions.insert(*func_id, (ir_func_id, signature));

                self.symbols.last_mut().unwrap().insert(*name, Symbol {
                    kind: SymbolKind::Func(ir_func_id),
                    ty: data.fty
                });
            },
            ExprKind::FunctionDef { name, params, body, return_ty, .. } => {
                let func_id = &self.function_decls[&node.id];
                if !self.ir_functions.contains_key(func_id) {
                    let data = &self.functions[func_id];
                    let mut signature = self.module.make_signature();
                    let target = self.module.isa().triple();
                    for (ty, param) in data.param_tys.iter().zip(params) {
                        signature.params.push(AbiParam::new(
                            self.types[ty].into_clif_type(target, self.path, param.ty.span, self.no_color)
                                .map_err(IrGenError::Diagnostic)?
                        ));
                    }
                    let data_ret_ty = &self.types[&data.ret_ty];
                    if *data_ret_ty != SemType::Unit {
                        signature.returns.push(AbiParam::new(
                            data_ret_ty.into_clif_type(target, self.path, return_ty.as_ref().unwrap().span, self.no_color)
                                .map_err(IrGenError::Diagnostic)?
                        ));
                    } else {
                        signature.returns.push(AbiParam::new(types::I8));
                    }

                    let string_name = if self.symbols.len() == 1 {
                        self.rodeo.resolve(name).to_string()
                    } else {
                        format!("{}_{}", self.rodeo.resolve(name), self.symbols.len())
                    };
                    let ir_func_id = self.module.declare_function(&string_name, Linkage::Export, &signature)
                        .map_err(|err| IrGenError::BackendError(err.into(), self.no_color))?;
                    self.ir_functions.insert(*func_id, (ir_func_id, signature));
                    
                    self.symbols.last_mut().unwrap().insert(*name, Symbol {
                        kind: SymbolKind::Func(ir_func_id),
                        ty: data.fty
                    });
                }

                let mut ctx = self.module.make_context();
                ctx.func = Function::with_name_signature(Default::default(), self.ir_functions[func_id].1.clone());
                let mut func_ctx = FunctionBuilderContext::new();

                {
                    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
                    let entry = builder.create_block();
                    builder.append_block_params_for_function_params(entry);
                    builder.switch_to_block(entry);
                    builder.seal_block(entry);
                    
                    let mut new_scope = HashMap::new();
                    for (idx, (param, &ty)) in params.iter().zip(&self.functions[func_id].param_tys).enumerate() {
                        if param.mutability {
                            let param_ty = &self.types[&ty];
                            let target = self.module.isa().triple();
                            let ss = builder.create_sized_stack_slot(StackSlotData {
                                kind: StackSlotKind::ExplicitSlot,
                                size: param_ty.size(target, self.path, param.name.1, self.no_color)
                                    .map_err(IrGenError::Diagnostic)?,
                                align_shift: param_ty.align(target, self.path, param.name.1, self.no_color)
                                    .map_err(IrGenError::Diagnostic)?,
                                key: None
                            });
                            let val = builder.block_params(entry)[idx];
                            builder.ins().stack_store(
                                val,
                                ss,
                                0
                            );
                            new_scope.insert(
                                param.name.0,
                                Symbol {
                                    kind: SymbolKind::SS(ss),
                                    ty
                                }
                            );
                        } else {
                            new_scope.insert(
                                param.name.0,
                                Symbol {
                                    kind: SymbolKind::Arg(builder.block_params(entry)[idx]),
                                    ty
                                }
                            );
                        }
                    }
                    self.symbols.push(new_scope);

                    let mut unit = builder.ins().iconst(types::I8, 0x0);
                    let final_val = self.walk_node(body, &mut builder, &mut unit)?;
                    builder.ins().return_(&[final_val]);
                }
                self.symbols.pop();

                self.module.define_function(self.ir_functions[func_id].0, &mut ctx)
                    .map_err(|err| IrGenError::BackendError(err.into(), self.no_color))?;
                self.module.clear_context(&mut ctx);
            },
            ExprKind::Semi(stmt) => self.walk_root_level_item(stmt)?,
            _ => unreachable!()
        }
        Ok(())
    }

    fn walk_node(&mut self, node: &Expr, builder: &mut FunctionBuilder, unit: &mut Value) -> Result<Value, IrGenError> {
        match &node.kind {
            ExprKind::Int(i) => Ok(builder.ins().iconst(
                self.types[&self.type_map[&node.id]]
                    .into_clif_type(self.module.isa().triple(), self.path, node.span, self.no_color)
                    .map_err(IrGenError::Diagnostic)?,
                *i
            )),
            ExprKind::Float(i) => match &self.types[&self.type_map[&node.id]] {
                SemType::F32 => Ok(builder.ins().f32const(*i as f32)),
                SemType::F64 | SemType::AmbiguousFloat => Ok(builder.ins().f64const(*i)),
                _ => unreachable!()
            },
            ExprKind::String(_s) => todo!(),
            ExprKind::Identifier(s) => {
                let symbol = self.find_ident(s).unwrap();

                match symbol.kind {
                    SymbolKind::SS(slot) => Ok(builder.ins()
                        .stack_load(
                            self.types[&symbol.ty]
                                .into_clif_type(self.module.isa().triple(), self.path, node.span, self.no_color)
                                .map_err(IrGenError::Diagnostic)?,
                            slot,
                            0)),
                    SymbolKind::Func(id) => {
                        let fref = self.module.declare_func_in_func(id, builder.func);
                        Ok(builder.ins().func_addr(
                            self.module.isa().pointer_type(),
                            fref
                        ))
                    },
                    SymbolKind::Arg(a) => Ok(a),
                }
            },
            ExprKind::Semi(stmt) => {
                self.walk_node(stmt, builder, unit)?;
                Ok(*unit)
            },
            ExprKind::Block(stmts) => {
                let mut final_val = *unit;
                for stmt in stmts {
                    final_val = self.walk_node(stmt, builder, unit)?;
                }
                Ok(final_val)
            },
            ExprKind::BinaryOp { lhs, rhs, op } => {
                if *op == Operator::Assign {
                    if let ExprKind::Identifier(s) = &lhs.kind {
                        let rval = self.walk_node(rhs, builder, unit)?;
                        match self.find_ident(s).unwrap().kind {
                            SymbolKind::SS(n) => {
                                builder.ins().stack_store(rval, n, 0);
                            },
                            SymbolKind::Func(_) => todo!(),
                            SymbolKind::Arg(_) => unreachable!(),
                        }
                        return Ok(rval);
                    } else { unreachable!() }
                }
                let lval = self.walk_node(lhs, builder, unit)?;
                let rval = self.walk_node(rhs, builder, unit)?;
                let lhs_ty = &self.types[&self.type_map[&lhs.id]];
                let diag = Diagnostic {
                    path: self.path.to_string(),
                    msg: "cannot infer type".to_string(),
                    span: node.span,
                    no_color: self.no_color,
                    secondaries: vec![],
                };
                match op {
                    Operator::Plus => if lhs_ty.is_int() {
                        Ok(builder.ins().iadd(lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fadd(lval, rval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Minus => if lhs_ty.is_int() {
                        Ok(builder.ins().isub(lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fsub(lval, rval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Star => if lhs_ty.is_int() {
                        Ok(builder.ins().imul(lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fmul(lval, rval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Slash => if lhs_ty.is_signed() {
                        Ok(builder.ins().sdiv(lval, rval))
                    } else if lhs_ty.is_unsigned() {
                        Ok(builder.ins().udiv(lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fadd(lval, rval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Modulo => if lhs_ty.is_signed() {
                        Ok(builder.ins().srem(lval, rval))
                    } else if lhs_ty.is_unsigned() {
                        Ok(builder.ins().urem(lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(frem(lval, rval, builder))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Eq => if lhs_ty.is_int() || *lhs_ty == SemType::Bool {
                        Ok(builder.ins().icmp(IntCC::Equal, lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fcmp(FloatCC::Equal, lval, rval))
                    } else if *lhs_ty == SemType::Unit {
                        Ok(builder.ins().iconst(types::I8, 1))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Ne => if lhs_ty.is_int() || *lhs_ty == SemType::Bool {
                        Ok(builder.ins().icmp(IntCC::NotEqual, lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fcmp(FloatCC::NotEqual, lval, rval))
                    } else if *lhs_ty == SemType::Unit {
                        Ok(builder.ins().iconst(types::I8, 1))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Gt => if lhs_ty.is_signed() {
                        Ok(builder.ins().icmp(IntCC::SignedGreaterThan, lval, rval))
                    } else if lhs_ty.is_unsigned() {
                        Ok(builder.ins().icmp(IntCC::UnsignedGreaterThan, lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fcmp(FloatCC::GreaterThan, lval, rval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Lt => if lhs_ty.is_signed() {
                        Ok(builder.ins().icmp(IntCC::SignedLessThan, lval, rval))
                    } else if lhs_ty.is_unsigned() {
                        Ok(builder.ins().icmp(IntCC::UnsignedLessThan, lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fcmp(FloatCC::LessThan, lval, rval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Ge => if lhs_ty.is_signed() {
                        Ok(builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lval, rval))
                    } else if lhs_ty.is_unsigned() {
                        Ok(builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lval, rval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Le => if lhs_ty.is_signed() {
                        Ok(builder.ins().icmp(IntCC::SignedLessThanOrEqual, lval, rval))
                    } else if lhs_ty.is_unsigned() {
                        Ok(builder.ins().icmp(IntCC::UnsignedLessThanOrEqual, lval, rval))
                    } else if lhs_ty.is_float() {
                        Ok(builder.ins().fcmp(FloatCC::LessThanOrEqual, lval, rval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    _ => unreachable!(),
                }
            },
            ExprKind::UnaryOp { operand, op } => {
                let oval = self.walk_node(operand, builder, unit)?;
                let operand_ty = &self.types[&self.type_map[&operand.id]];
                let diag = Diagnostic {
                    path: self.path.to_string(),
                    msg: "cannot infer type".to_string(),
                    span: node.span,
                    no_color: self.no_color,
                    secondaries: vec![],
                };
                match op {
                    Operator::Plus => Ok(oval),
                    Operator::Minus => if operand_ty.is_int() {
                        Ok(builder.ins().ineg(oval))
                    } else if operand_ty.is_float() {
                        Ok(builder.ins().fneg(oval))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    Operator::Bang => if operand_ty.is_int() {
                        Ok(builder.ins().bnot(oval))
                    } else if *operand_ty == SemType::Bool {
                        Ok(builder.ins().bxor_imm(oval, 1))
                    } else {
                        return Err(IrGenError::Diagnostic(diag));
                    },
                    _ => unreachable!(),
                }
            },
            ExprKind::Let { name, init, .. }
            | ExprKind::Var { name, init, .. } => {
                let ty = self.type_map[&node.id];
                let target = self.module.isa().triple();
                let slot = builder.create_sized_stack_slot(StackSlotData {
                    kind: StackSlotKind::ExplicitSlot,
                    size: self.types[&ty]
                        .size(target, self.path, node.span, self.no_color)
                        .map_err(IrGenError::Diagnostic)?,
                    align_shift: self.types[&ty]
                        .align(target, self.path, node.span, self.no_color)
                        .map_err(IrGenError::Diagnostic)?,
                    key: None
                });
                self.symbols.last_mut().unwrap().insert(*name, Symbol { kind: SymbolKind::SS(slot), ty });
                let ival = self.walk_node(init, builder, unit)?;
                builder.ins().stack_store(ival, slot, 0);
                Ok(ival)
            },
            ExprKind::If { condition, then_body, else_body } => {
                let cval = self.walk_node(condition, builder, unit)?;

                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge_block = builder.create_block();

                builder.append_block_param(
                    then_block,
                    types::I8
                );
                builder.append_block_param(
                    else_block,
                    types::I8
                );
                builder.append_block_param(
                    merge_block,
                    self.types[&self.type_map[&then_body.id]]
                        .into_clif_type(self.module.isa().triple(), self.path, then_body.span, self.no_color)
                        .map_err(IrGenError::Diagnostic)?
                );
                builder.append_block_param(
                    merge_block,
                    types::I8
                );

                builder.ins().brif(
                    cval,
                    then_block,
                    &[BlockArg::Value(*unit)],
                    else_block,
                    &[BlockArg::Value(*unit)]
                );

                builder.switch_to_block(then_block);
                let mut then_unit = builder.block_params(then_block)[0];
                self.symbols.push(HashMap::new());
                let then_value = self.walk_node(then_body, builder, &mut then_unit)?;
                self.symbols.pop();
                builder.ins().jump(
                    merge_block,
                    &[BlockArg::Value(then_value), BlockArg::Value(then_unit)]
                );

                builder.switch_to_block(else_block);
                let mut else_unit = builder.block_params(else_block)[0];
                let else_value = match else_body {
                    Some(expr) => {
                        self.symbols.push(HashMap::new());
                        let val = self.walk_node(expr, builder, &mut else_unit)?;
                        self.symbols.pop();
                        val
                    },
                    None => else_unit,
                };
                builder.ins().jump(
                    merge_block,
                    &[BlockArg::Value(else_value), BlockArg::Value(else_unit)]
                );

                builder.seal_block(then_block);
                builder.seal_block(else_block);
                builder.seal_block(merge_block);

                builder.switch_to_block(merge_block);
                *unit = builder.block_params(merge_block)[1];
                Ok(builder.block_params(merge_block)[0])
            },
            ExprKind::While { condition, body, cont_expr } => {
                let cond_eval_block = builder.create_block();
                let body_block = builder.create_block();
                let break_block = builder.create_block();
                
                builder.append_block_param(
                    cond_eval_block,
                    types::I8
                );
                builder.append_block_param(
                    body_block,
                    types::I8
                );
                builder.append_block_param(
                    break_block,
                    types::I8
                );

                builder.ins().jump(
                    cond_eval_block,
                    &[BlockArg::Value(*unit)],
                );

                builder.switch_to_block(cond_eval_block);
                let cval = self.walk_node(condition, builder, unit)?;
                let cond_unit = builder.block_params(cond_eval_block)[0];
                builder.ins().brif(cval,
                    body_block,
                    &[BlockArg::Value(cond_unit)],
                    break_block,
                    &[BlockArg::Value(cond_unit)]
                );

                builder.switch_to_block(body_block);
                let mut body_unit = builder.block_params(body_block)[0];
                self.symbols.push(HashMap::new());
                self.walk_node(body, builder, &mut body_unit)?;
                self.symbols.pop();
                if let Some(expr) = cont_expr {
                    self.symbols.push(HashMap::new());
                    self.walk_node(expr, builder, &mut body_unit)?;
                    self.symbols.pop();
                }
                builder.ins().jump(
                    cond_eval_block,
                    &[BlockArg::Value(*unit)]
                );
                
                builder.seal_block(cond_eval_block);
                builder.seal_block(body_block);
                builder.seal_block(break_block);

                builder.switch_to_block(break_block);
                *unit = builder.block_params(break_block)[0];

                Ok(*unit)
            },
            ExprKind::FunctionDecl { .. } | ExprKind::FunctionDef { .. } => {
                self.walk_root_level_item(node)?;
                Ok(*unit)
            },
            ExprKind::FunctionCall { callee, args } => {
                let fn_ptr = self.walk_node(callee, builder, unit)?;
                let mut arg_vals = vec![];
                for arg in args {
                    arg_vals.push(self.walk_node(arg, builder, unit)?);
                }
                let mut sig = Signature::new(self.module.isa().default_call_conv());
                let target = self.module.isa().triple();
                if let SemType::Function(_, params, ret, _) = &self.types[&self.type_map[&callee.id]] {
                    for param in params {
                        sig.params.push(AbiParam::new(
                            self.types[param]
                                .into_clif_type(target, self.path, node.span, self.no_color) // we can't error here
                                .map_err(IrGenError::Diagnostic)?,
                        ));
                    }
                    sig.returns.push(AbiParam::new(
                        self.types[ret].into_clif_type(target, self.path, node.span, self.no_color) // we can't error here
                            .map_err(IrGenError::Diagnostic)?,
                    ));
                }
                let sig_ref = builder.import_signature(sig);
                let call = builder.ins().call_indirect(sig_ref, fn_ptr, &arg_vals);
                Ok(builder.inst_results(call)[0])
            },
        }
    }
}

fn frem(x: Value, y: Value, builder: &mut FunctionBuilder) -> Value {
    let v0 = builder.ins().fdiv(x, y);
    let v1 = builder.ins().trunc(v0);
    let v3 = builder.ins().fmul(y, v1);
    builder.ins().fsub(x, v3)
}

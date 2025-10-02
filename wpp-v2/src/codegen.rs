use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::{Linkage, Module},
    types::BasicTypeEnum,
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue},
    AddressSpace,
    OptimizationLevel,
};
use std::collections::HashMap;
use inkwell::types::BasicType;
use inkwell::types::BasicMetadataTypeEnum;


use crate::ast::{Expr, Node};

/// The core LLVM code generator for W++
pub struct Codegen <'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub i32_type: inkwell::types::IntType<'ctx>,

    /// Symbol table: name -> (alloca ptr, type of the slot)
    pub vars: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
        pub loop_stack: Vec<(inkwell::basic_block::BasicBlock<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)>,
    pub switch_stack: Vec<inkwell::basic_block::BasicBlock<'ctx>>,
    exception_flag: Option<PointerValue<'ctx>>,
    exception_value_i32: Option<PointerValue<'ctx>>,
    exception_value_str: Option<PointerValue<'ctx>>,
        pub functions: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
    let module = context.create_module(name);
    let builder = context.create_builder();
    let i32_type = context.i32_type();
    let bool_type = context.bool_type();
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());

    // Create global flags and exception storage
    let flag = module.add_global(bool_type, None, "_exception_flag");
    flag.set_initializer(&bool_type.const_int(0, false));

    let exc_i32 = module.add_global(i32_type, None, "_exception_value_i32");
    exc_i32.set_initializer(&i32_type.const_int(0, false));

    let exc_str = module.add_global(i8_ptr_type, None, "_exception_value_str");
    exc_str.set_initializer(&i8_ptr_type.const_null());

    Self {
        context,
        module,
        builder,
        i32_type,
        vars: HashMap::new(),
        loop_stack: Vec::new(),
        switch_stack: Vec::new(),
        exception_flag: Some(flag.as_pointer_value()),
        exception_value_i32: Some(exc_i32.as_pointer_value()),
        exception_value_str: Some(exc_str.as_pointer_value()),
            functions: HashMap::new(), // ✅
    }
}


    pub fn create_engine(&self) -> ExecutionEngine<'ctx> {
        self.module
            .create_jit_execution_engine(OptimizationLevel::None)
            .expect("Failed to create JIT engine")
    }

    /// Compile an expression to a BasicValue (either i32 or i8* for now).
    pub fn compile_expr(&mut self, expr: &Expr) -> BasicValueEnum<'ctx> {
    match expr {
        // === Integer literal ===
        Expr::Literal(value) => self.i32_type.const_int(*value as u64, false).into(),

        // === String literal ===
        // === String literal ===
Expr::StringLiteral(s) => {
    // Create a null-terminated string constant
    let str_val = self.context.const_string(s.as_bytes(), true);

    // Create a unique global name (optional but avoids collisions)
    static mut STRING_ID: usize = 0;
    let name = unsafe {
        let id = STRING_ID;
        STRING_ID += 1;
        format!("strlit_{}", id)
    };

    // Add it as a global constant
    let global = self.module.add_global(str_val.get_type(), None, &name);
    global.set_initializer(&str_val);
    global.set_constant(true);
    global.set_linkage(inkwell::module::Linkage::Private);

    global.as_pointer_value().into()
}



        // === Variable lookup ===
        Expr::Variable(name) => {
            let (ptr, ty) = self
                .vars
                .get(name)
                .unwrap_or_else(|| panic!("Unknown variable '{}'", name));
            self.builder.build_load(*ty, *ptr, &format!("load_{}", name)).unwrap().into()
        }

        // === Binary operation ===
        Expr::BinaryOp { left, op, right } => {
    if op == "=" {
        if let Expr::Variable(var_name) = left.as_ref() {
            // Compute RHS
            let value = self.compile_expr(right.as_ref());

            // Lookup variable and store
            if let Some((ptr, _)) = self.vars.get(var_name) {
                self.builder.build_store(*ptr, value).unwrap();

                // ✅ Ensure the result is always an i32 (if not already)
                let result = match value {
                    BasicValueEnum::IntValue(iv) => iv,
                    BasicValueEnum::PointerValue(_) => {
                        // Assigning pointer types not supported yet
                        self.i32_type.const_int(0, false)
                    }
                    _ => self.i32_type.const_int(0, false),
                };

                return result.into();
            } else {
                panic!("Unknown variable in assignment: {}", var_name);
            }
        } else {
            panic!("Left-hand side of assignment must be a variable");
        }
    }

    // === Arithmetic and comparison ===
    let left_raw = self.compile_expr(left.as_ref());
let right_raw = self.compile_expr(right.as_ref());

// Safety: ensure both sides are IntValues before doing math
let (left_val, right_val) = match (left_raw, right_raw) {
    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => (l, r),
    _ => panic!("Binary operator `{}` only supports integers", op),
};


    let result = match op.as_str() {
        "+" => self.builder.build_int_add(left_val, right_val, "addtmp").unwrap().into(),
        "-" => self.builder.build_int_sub(left_val, right_val, "subtmp").unwrap().into(),
        "*" => self.builder.build_int_mul(left_val, right_val, "multmp").unwrap().into(),
        "/" => self.builder.build_int_signed_div(left_val, right_val, "divtmp").unwrap().into(),
        "==" => self.builder.build_int_compare(inkwell::IntPredicate::EQ, left_val, right_val, "eqtmp").unwrap().into(),
        "!=" => self.builder.build_int_compare(inkwell::IntPredicate::NE, left_val, right_val, "netmp").unwrap().into(),
        "<"  => self.builder.build_int_compare(inkwell::IntPredicate::SLT, left_val, right_val, "lttmp").unwrap().into(),
        "<=" => self.builder.build_int_compare(inkwell::IntPredicate::SLE, left_val, right_val, "letmp").unwrap().into(),
        ">"  => self.builder.build_int_compare(inkwell::IntPredicate::SGT, left_val, right_val, "gttmp").unwrap().into(),
        ">=" => self.builder.build_int_compare(inkwell::IntPredicate::SGE, left_val, right_val, "getmp").unwrap().into(),
        _ => panic!("Unsupported binary operator: {}", op),
    };

    // ✅ Auto-convert i1 → i32 for arithmetic safety
    match result {
        BasicValueEnum::IntValue(iv) if iv.get_type().get_bit_width() == 1 => {
            self.builder.build_int_z_extend(iv, self.i32_type, "bool_to_i32").unwrap().into()
        }
        _ => result,
    }
}




        // === Boolean literal ===
        Expr::BoolLiteral(value) => self.context.bool_type().const_int(*value as u64, false).into(),

        // === Function call (print, etc.) ===
        Expr::Call { name, args } => {
            if name == "print" {
    // === Declare printf and puts if not yet defined ===
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());

    let printf = self.module.get_function("printf").unwrap_or_else(|| {
        let ty = self.context.i32_type().fn_type(&[i8ptr.into()], true);
        self.module.add_function("printf", ty, None)
    });

    let puts = self.module.get_function("puts").unwrap_or_else(|| {
        let ty = self.context.i32_type().fn_type(&[i8ptr.into()], false);
        self.module.add_function("puts", ty, None)
    });

    // === Compile the argument ===
    if args.is_empty() {
        panic!("print() expects 1 argument");
    }
    let val = self.compile_expr(&args[0]);

    match val {
        // === Print string ===
        BasicValueEnum::PointerValue(pv) => {
            // Null-pointer check (optional safety)
            let fmt = self.builder.build_global_string_ptr("%s\n", "fmt_s").unwrap();
            let call_args: Vec<BasicMetadataValueEnum> =
                vec![fmt.as_pointer_value().into(), pv.into()];
            self.builder.build_call(printf, &call_args, "call_printf_str").unwrap();
        }

        // === Print int or bool ===
        BasicValueEnum::IntValue(iv) => {
            if iv.get_type().get_bit_width() == 1 {
                // ✅ Boolean print
                let cond = self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    iv,
                    self.context.bool_type().const_int(1, false),
                    "is_true",
                ).unwrap();

                let true_str = self.builder.build_global_string_ptr("true\n", "true_str").unwrap();
                let false_str = self.builder.build_global_string_ptr("false\n", "false_str").unwrap();

                let parent_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let then_block = self.context.append_basic_block(parent_fn, "then");
                let else_block = self.context.append_basic_block(parent_fn, "else");
                let merge_block = self.context.append_basic_block(parent_fn, "merge");

                self.builder.build_conditional_branch(cond, then_block, else_block).unwrap();

                // THEN block
                self.builder.position_at_end(then_block);
                self.builder
                    .build_call(puts, &[true_str.as_pointer_value().into()], "puts_true")
                    .unwrap();
                self.builder.build_unconditional_branch(merge_block).unwrap();

                // ELSE block
                self.builder.position_at_end(else_block);
                self.builder
                    .build_call(puts, &[false_str.as_pointer_value().into()], "puts_false")
                    .unwrap();
                self.builder.build_unconditional_branch(merge_block).unwrap();

                // MERGE block
                self.builder.position_at_end(merge_block);
            } else {
                // === Normal integer print ===
                let fmt = self.builder.build_global_string_ptr("%d\n", "fmt_d").unwrap();
                let args = vec![fmt.as_pointer_value().into(), iv.into()];
                self.builder.build_call(printf, &args, "call_printf_int").unwrap();
            }
        }

        // === Unsupported type ===
        other => panic!("print(..): unsupported value type {:?}", other),
    }

    // Print returns dummy int (printf returns int)
    self.i32_type.const_int(0, false).into()
}

            else if let Some(func_val) = self.functions.get(name).cloned() {
    let func = func_val; // owned copy (FunctionValue is Copy internally)

    // Compile arguments
    let mut compiled_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
    for arg in args {
        let val = self.compile_expr(arg);
        compiled_args.push(val.into());
    }

    // Call the LLVM function
    let call_site = self.builder
        .build_call(func, &compiled_args, &format!("call_{}", name))
        .unwrap();

    // Return the result (functions return i32)
    call_site.try_as_basic_value().left().unwrap_or_else(|| {
        self.i32_type.const_int(0, false).into()
    })
}

            else {
                panic!("Unknown function: {}", name);
            }
        }

        // === If expression ===
        Expr::If { cond, then_branch, else_branch } => {
    let cond_val = self.compile_expr(cond.as_ref());
    let cond_i1 = match cond_val {
        BasicValueEnum::IntValue(iv) => {
            if iv.get_type().get_bit_width() == 1 {
                iv
            } else {
                self.builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        iv,
                        self.i32_type.const_int(0, false),
                        "if_cond_cmp",
                    )
                    .unwrap()
            }
        }
        _ => panic!("Condition in if must be int or bool"),
    };

    // Create blocks
    let parent_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let then_bb = self.context.append_basic_block(parent_fn, "if_then");
    let else_bb = self.context.append_basic_block(parent_fn, "if_else");
    let end_bb  = self.context.append_basic_block(parent_fn, "if_end");

    // Conditional branch
    self.builder.build_conditional_branch(cond_i1, then_bb, else_bb).unwrap();

    // THEN branch
    self.builder.position_at_end(then_bb);
    for node in then_branch {
        self.compile_node(node);
    }
    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
        self.builder.build_unconditional_branch(end_bb).unwrap();
    }

    // ELSE branch
    self.builder.position_at_end(else_bb);
    if let Some(else_nodes) = else_branch {
        for node in else_nodes {
            self.compile_node(node);
        }
    }
    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
        self.builder.build_unconditional_branch(end_bb).unwrap();
    }

    // MERGE block
    self.builder.position_at_end(end_bb);

    // ✅ No restoring — builder now correctly positioned
    self.i32_type.const_int(0, false).into()
}




        Expr::While { cond, body } => {
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();

    // === Define core blocks ===
    let cond_bb = self.context.append_basic_block(func, "while_cond");
    let body_bb = self.context.append_basic_block(func, "while_body");
    let end_bb  = self.context.append_basic_block(func, "while_end");
    let after_bb = self.context.append_basic_block(func, "after_loop");

    // Push (continue_target, break_target) → continue jumps to cond_bb directly
    self.loop_stack.push((cond_bb, end_bb));

    // Jump to condition first
    self.builder.build_unconditional_branch(cond_bb).unwrap();

    // === Condition ===
    self.builder.position_at_end(cond_bb);
    let cond_val = self.compile_expr(cond);
    let cond_i1 = match cond_val {
        BasicValueEnum::IntValue(iv) if iv.get_type().get_bit_width() == 1 => iv,
        BasicValueEnum::IntValue(iv) => self.builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                iv,
                self.i32_type.const_int(0, false),
                "while_cond_cmp"
            )
            .unwrap(),
        _ => panic!("while condition must be int/bool"),
    };

    // Branch to body or exit
    self.builder.build_conditional_branch(cond_i1, body_bb, end_bb).unwrap();

    // === Body ===
    self.builder.position_at_end(body_bb);
    for stmt in body {
        self.compile_node(stmt);
    }

    // If body didn’t end with a terminator (break/continue), loop back to condition
    self.safe_branch(cond_bb);

    // === End (break target) ===
    self.builder.position_at_end(end_bb);
    self.loop_stack.pop();
    self.safe_branch(after_bb);

    // === After loop (execution continues here) ===
    self.builder.position_at_end(after_bb);
    self.i32_type.const_int(0, false).into()
}










Expr::For { init, cond, post, body } => {
    if let Some(init_node) = init {
        self.compile_node(init_node);
    }

    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let cond_bb = self.context.append_basic_block(func, "for_cond");
    let body_bb = self.context.append_basic_block(func, "for_body");
    let post_bb = self.context.append_basic_block(func, "for_post");
    let end_bb  = self.context.append_basic_block(func, "for_end");

    // 🧩 Push loop context (for break/continue)
    self.loop_stack.push((post_bb, end_bb));

    // === Jump to condition
    self.builder.build_unconditional_branch(cond_bb).unwrap();

    // === Condition block
    self.builder.position_at_end(cond_bb);
    let cond_val = if let Some(c) = cond {
        self.compile_expr(c).into_int_value()
    } else {
        self.context.bool_type().const_int(1, false)
    };
    self.builder.build_conditional_branch(cond_val, body_bb, end_bb).unwrap();

    // === Body block
    self.builder.position_at_end(body_bb);
    for stmt in body {
        self.compile_node(stmt);
    }

    // If body didn’t end with break/continue, jump to post
    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
        self.builder.build_unconditional_branch(post_bb).unwrap();
    }

    // === Post block
    self.builder.position_at_end(post_bb);
    if let Some(post_expr) = post {
    match post_expr.as_ref() {
        Expr::BinaryOp { left, op, right } if op == "=" => {
            if let Expr::Variable(var_name) = left.as_ref() {
                let value = self.compile_expr(right.as_ref());
                if let Some((ptr, _)) = self.vars.get(var_name) {
                    self.builder.build_store(*ptr, value).unwrap();
                } else {
                    panic!("Unknown variable in for-loop post: {}", var_name);
                }
            }
        }
        _ => {
            // ✅ Wrap Expr in Node::Expr to reuse compile_node
            self.compile_node(&Node::Expr(post_expr.as_ref().clone()));
        }
    }
}


    // Jump back to condition if not terminated
    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
        self.builder.build_unconditional_branch(cond_bb).unwrap();
    }

    // === End block
    self.builder.position_at_end(end_bb);

    // 🧩 Pop loop context
    self.loop_stack.pop();

    // ✅ Finalize: ensure valid builder position after loop
    let after_loop = self.context.append_basic_block(func, "after_loop");
    self.builder.build_unconditional_branch(after_loop).ok();
    self.builder.position_at_end(after_loop);

    self.context.i32_type().const_int(0, false).into()
}





Expr::Break => {
    if let Some((_, break_target)) = self.loop_stack.last() {
        // Break inside loop
        self.safe_branch(*break_target);
    } else if let Some(switch_end) = self.switch_stack.last() {
        // ✅ Break inside switch
        self.safe_branch(*switch_end);
    } else {
        panic!("'break' used outside of loop or switch");
    }

    // Move to dummy block after break
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let dummy = self.context.append_basic_block(func, "after_break");
    self.builder.position_at_end(dummy);

    self.i32_type.const_int(0, false).into()
}


Expr::Continue => {
    if let Some((cont_target, _)) = self.loop_stack.last() {
        // Jump to continue target
        self.builder.build_unconditional_branch(*cont_target).unwrap();

        // ✅ Move builder to a new dummy unreachable block (so codegen continues safely)
        let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let unreachable_block = self.context.append_basic_block(func, "after_continue_unreachable");
        self.builder.position_at_end(unreachable_block);
    } else {
        panic!("'continue' used outside of loop");
    }

    self.i32_type.const_int(0, false).into()
}


Expr::Switch { expr, cases, default } => {
    self.compile_switch(expr, cases, default)
}
Expr::Throw { expr } => {
    // Evaluate the thrown expression
    let value = self.compile_expr(expr);

    // Determine if it’s a string (pointer) or integer
    match value {
        BasicValueEnum::PointerValue(pv) => {
            // Throwing a string: store string in exception_value_str
            if let Some(val_str) = self.exception_value_str {
                self.builder.build_store(val_str, pv).unwrap();
            } else {
                panic!("throw: no active string exception slot");
            }

            // Set flag to 1 (string-type exception)
            if let Some(flag) = self.exception_flag {
                self.builder.build_store(flag, self.i32_type.const_int(2, false)).unwrap();
            }
        }

        BasicValueEnum::IntValue(iv) => {
            // Throwing an integer: store in exception_value_i32
            if let Some(val_i32) = self.exception_value_i32 {
                self.builder.build_store(val_i32, iv).unwrap();
            } else {
                panic!("throw: no active int exception slot");
            }

            // Set flag to 1 (integer-type exception)
            if let Some(flag) = self.exception_flag {
                self.builder.build_store(flag, self.i32_type.const_int(1, false)).unwrap();
            }
        }

        _ => panic!("Unsupported throw type"),
    }

    // Jump directly to end of try block — we’ll let TryCatch handle propagation
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let end_bb = self.context.append_basic_block(func, "throw_exit");
    self.builder.build_unconditional_branch(end_bb).unwrap();
    self.builder.position_at_end(end_bb);

    // Return dummy value
    self.i32_type.const_int(0, false).into()
}




Expr::TryCatch { try_block, catch_var, catch_block, finally_block } => {
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();

    // === Create blocks ===
    let try_bb = self.context.append_basic_block(func, "try_block");
    let catch_bb = self.context.append_basic_block(func, "catch_block");
    let finally_bb = self.context.append_basic_block(func, "finally_block");
    let end_bb = self.context.append_basic_block(func, "try_end");

    // === Preserve old exception state ===
    let old_flag = self.exception_flag;
    let old_val_i32 = self.exception_value_i32;
    let old_val_str = self.exception_value_str;

    // === Allocate local exception slots ===
    let flag_ptr = self.builder.build_alloca(self.i32_type, "exc_flag_local").unwrap();
    self.builder.build_store(flag_ptr, self.i32_type.const_int(0, false)).unwrap();

    let val_i32_ptr = self.builder.build_alloca(self.i32_type, "exc_val_i32_local").unwrap();
    self.builder.build_store(val_i32_ptr, self.i32_type.const_int(0, false)).unwrap();

    let str_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
    let val_str_ptr = self.builder.build_alloca(str_ptr_ty, "exc_val_str_local").unwrap();
    self.builder.build_store(val_str_ptr, str_ptr_ty.const_null()).unwrap();

    self.exception_flag = Some(flag_ptr);
    self.exception_value_i32 = Some(val_i32_ptr);
    self.exception_value_str = Some(val_str_ptr);

    // === Jump into try ===
    self.builder.build_unconditional_branch(try_bb).unwrap();

    // --- TRY block ---
    self.builder.position_at_end(try_bb);
    for node in try_block {
        self.compile_node(node);
    }

    // Read flag
    let flag_val = self.builder.build_load(self.i32_type, flag_ptr, "flag").unwrap().into_int_value();
    let cond = self.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        flag_val,
        self.i32_type.const_int(1, false),
        "is_throw",
    ).unwrap();

    self.builder.build_conditional_branch(cond, catch_bb, finally_bb).unwrap();

    // --- CATCH block ---
self.builder.position_at_end(catch_bb);
if let Some(var) = catch_var {
    let str_ty = self.context.i8_type().ptr_type(AddressSpace::default());
    let alloca = self.builder.build_alloca(str_ty, var).unwrap();
    let val_str = self.builder.build_load(str_ty, val_str_ptr, "ex_str").unwrap();
    self.builder.build_store(alloca, val_str).unwrap();
    self.vars.insert(var.clone(), (alloca, str_ty.as_basic_type_enum()));
}
for node in catch_block {
    self.compile_node(node);
}
// reset flag after catch
self.builder.build_store(flag_ptr, self.i32_type.const_int(0, false)).unwrap();

// 🔹 instead of jumping directly to finally, go to a neutral after_catch block
let after_catch_bb = self.context.append_basic_block(func, "after_catch");
self.builder.build_unconditional_branch(after_catch_bb).unwrap();

// --- AFTER_CATCH -> FINALLY ---
self.builder.position_at_end(after_catch_bb);
self.builder.build_unconditional_branch(finally_bb).unwrap();

// --- FINALLY block ---
self.builder.position_at_end(finally_bb);
if let Some(fb) = finally_block {
    for node in fb {
        self.compile_node(node);
    }
}
// always branch to end, but only if block not already terminated
self.safe_branch(end_bb);

// --- END ---
self.builder.position_at_end(end_bb);

// === Restore previous exception pointers ===
self.exception_flag = old_flag;
self.exception_value_i32 = old_val_i32;
self.exception_value_str = old_val_str;

self.i32_type.const_int(0, false).into()

}







Expr::Funcy { name, params, body } => {
    self.compile_funcy(name, params, body);
    self.i32_type.const_int(0, false).into()
}



Expr::Return(expr_opt) => {
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();

    // Determine return value
    let ret_val = if let Some(expr) = expr_opt {
    let v = self.compile_expr(expr);
    match v {
        BasicValueEnum::IntValue(iv) => iv,
        BasicValueEnum::PointerValue(pv) => {
            // Just print the string for now and return 0
            let printf = self.module.get_function("printf").unwrap_or_else(|| {
                let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
                let ty = self.context.i32_type().fn_type(&[i8ptr.into()], true);
                self.module.add_function("printf", ty, None)
            });
            let fmt = self.builder.build_global_string_ptr("%s\n", "ret_fmt").unwrap();
            let args = vec![fmt.as_pointer_value().into(), pv.into()];
            self.builder.build_call(printf, &args, "print_return").unwrap();
            self.i32_type.const_int(0, false)
        }
        _ => self.i32_type.const_int(0, false),
    }
} else {
    self.i32_type.const_int(0, false)
};


    // Build actual LLVM return
    self.builder.build_return(Some(&ret_val)).unwrap();

    // Move builder to a safe new block to prevent further codegen errors
    let after_ret = self.context.append_basic_block(func, "after_return");
    self.builder.position_at_end(after_ret);

    self.i32_type.const_int(0, false).into()
}








        // === Fallback ===
        _ => panic!("Unhandled expression: {:?}", expr),
    }
}

    /// Compile a statement. Returns last expression value (if any).
    pub fn compile_node(&mut self, node: &Node) -> Option<BasicValueEnum<'ctx>> {
    // Don’t compile if the current block already has a terminator
    if let Some(block) = self.builder.get_insert_block() {
        if block.get_terminator().is_some() {
            return None;
        }
    }

    match node {
        Node::Let { name, value } => {
            let v = self.compile_expr(value);

            let (alloca, ty): (PointerValue, BasicTypeEnum) = match v {
                BasicValueEnum::IntValue(_) => {
                    let ty = self.i32_type.as_basic_type_enum();
                    let p = self.builder.build_alloca(ty, name).unwrap();
                    (p, ty)
                }
                BasicValueEnum::PointerValue(_) => {
                    let ty = self
                        .context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum();
                    let p = self.builder.build_alloca(ty, name).unwrap();
                    (p, ty)
                }
                other => panic!("let {} = <unsupported type: {:?}>", name, other),
            };

            self.builder.build_store(alloca, v).unwrap();
            self.vars.insert(name.clone(), (alloca, ty));
            None
        }

        Node::Expr(expr) => {
            let v = self.compile_expr(expr);

            // ✅ Stop if this expression emitted a terminator (e.g. break/continue)
            if let Some(block) = self.builder.get_insert_block() {
                if block.get_terminator().is_some() {
                    return None;
                }
            }

            Some(v)
        }
    }
}




    /// Create main(), compile nodes, and return i32.
    pub fn compile_main(&mut self, nodes: &[Node]) -> FunctionValue<'ctx> {
    let fn_type = self.i32_type.fn_type(&[], false);
    let function = self.module.add_function("main", fn_type, None);
    let entry = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(entry);

    // === Allocate exception slots === (same as before)
    let flag_ptr = self.builder.build_alloca(self.i32_type, "exc_flag").unwrap();
    self.builder.build_store(flag_ptr, self.i32_type.const_int(0, false)).unwrap();

    let val_i32_ptr = self.builder.build_alloca(self.i32_type, "exc_val_i32").unwrap();
    self.builder.build_store(val_i32_ptr, self.i32_type.const_int(0, false)).unwrap();

    let str_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
    let val_str_ptr = self.builder.build_alloca(str_ptr_ty, "exc_val_str").unwrap();
    self.builder.build_store(val_str_ptr, str_ptr_ty.const_null()).unwrap();

    self.exception_flag = Some(flag_ptr);
    self.exception_value_i32 = Some(val_i32_ptr);
    self.exception_value_str = Some(val_str_ptr);

    // === Pass 1: Predeclare all function signatures ===
for node in nodes {
    if let Node::Expr(Expr::Funcy { name, params, .. }) = node {
        // Build param types (we can reuse scan_for_ints here)
        let mut int_params: std::collections::HashSet<String> = std::collections::HashSet::new();
        fn scan_for_ints(nodes: &[Node], int_params: &mut std::collections::HashSet<String>) {
            for node in nodes {
                match node {
                    Node::Expr(expr) => match expr {
                        crate::ast::Expr::BinaryOp { left, right, .. } => {
                            if let crate::ast::Expr::Variable(name) = left.as_ref() {
                                int_params.insert(name.clone());
                            }
                            if let crate::ast::Expr::Variable(name) = right.as_ref() {
                                int_params.insert(name.clone());
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        // for now: assume all params are i32 (safe default)
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> =
            params.iter().map(|_| self.i32_type.into()).collect();

        let fn_type = self.i32_type.fn_type(&param_types, false);
        let function = self.module.add_function(name, fn_type, None);
        self.functions.insert(name.clone(), function);
    }
}

// === Pass 2: Compile function bodies ===
for node in nodes {
    if let Node::Expr(Expr::Funcy { name, params, body }) = node {
        self.compile_funcy(name, params, body);
    }
}


    // === Now compile regular statements (calls, control flow, etc.)
    let mut last_int: Option<IntValue> = None;
    for node in nodes {
        if let Some(v) = self.compile_node(node) {
            if let BasicValueEnum::IntValue(iv) = v {
                last_int = Some(iv);
            }
        }
    }

    self.ensure_builder_position();
    let ret_val = last_int.unwrap_or_else(|| self.i32_type.const_int(0, false));
    self.builder.build_return(Some(&ret_val)).unwrap();
    function
}



    pub fn run_jit(&self) {
    let engine = self.create_engine();

    unsafe {
        // === Cross-platform printf registration ===
        use libc::printf as c_printf; // works on all OSes, including Windows

        if let Some(func) = self.module.get_function("printf") {
            engine.add_global_mapping(&func, c_printf as usize);
        }

        // === Execute main() ===
        let addr = engine
            .get_function_address("main")
            .expect("Failed to get main() address");
        let main_fn: unsafe extern "C" fn() -> i32 = std::mem::transmute(addr);
        let result = main_fn();

        println!("✅ JIT result from main(): {}", result);
    }
}


fn compile_switch(
    &mut self,
    discr_expr: &Expr,
    cases: &Vec<(Expr, Vec<Node>)>,
    default: &Option<Vec<Node>>,
) -> BasicValueEnum<'ctx> {
    let discr_val = self.compile_expr(discr_expr).into_int_value();

    let function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let end_bb = self.context.append_basic_block(function, "switch.end");
    self.switch_stack.push(end_bb);
    let default_bb = self.context.append_basic_block(function, "switch.default");

    // Prepare case blocks
    let mut case_blocks = Vec::new();
    for (case_expr, body) in cases {
        let bb = self.context.append_basic_block(function, "switch.case");
        case_blocks.push((case_expr.clone(), bb, body));
    }

    // Compile expressions before switch instruction
    let mut case_values = Vec::new();
    for (case_expr, bb, _) in &case_blocks {
        let val = self.compile_expr(case_expr);
        case_values.push((val.into_int_value(), *bb));
    }

    // Build switch instruction
    self.builder
        .build_switch(discr_val, default_bb, &case_values)
        .unwrap();

    // Compile each case body
    for (_, bb, body) in &case_blocks {
        self.builder.position_at_end(*bb);
        for node in *body {
            self.compile_node(node);
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(end_bb).unwrap();
        }
    }

    // Default body
    self.builder.position_at_end(default_bb);
    if let Some(body) = default {
        for node in body {
            self.compile_node(node);
        }
    }
    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
        self.builder.build_unconditional_branch(end_bb).unwrap();
    }

    // Return to end
    self.builder.position_at_end(end_bb);
    self.switch_stack.pop();
    self.context.i32_type().const_int(0, false).into()
}
pub fn compile_funcy(
    &mut self,
    name: &str,
    params: &[String],
    body: &[Node],
) -> FunctionValue<'ctx> {
    // 🧠 Infer which parameters are used in math (so they’re i32)
    let mut int_params = std::collections::HashSet::new();

    fn scan_for_ints(nodes: &[Node], int_params: &mut std::collections::HashSet<String>) {
    for node in nodes {
        match node {
            Node::Expr(expr) => match expr {
                Expr::BinaryOp { left, right, .. } => {
                    if let Expr::Variable(name) = left.as_ref() {
                        int_params.insert(name.clone());
                    }
                    if let Expr::Variable(name) = right.as_ref() {
                        int_params.insert(name.clone());
                    }
                    // Recurse deeper if operands contain nested expressions
                    scan_for_ints(&[Node::Expr(*left.clone())], int_params);
                    scan_for_ints(&[Node::Expr(*right.clone())], int_params);
                }

                Expr::Return(inner) => {
                    if let Some(inner_expr) = inner {
                        scan_for_ints(&[Node::Expr(*inner_expr.clone())], int_params);
                    }
                }

                Expr::If { cond, then_branch, else_branch } => {
                    scan_for_ints(&[Node::Expr(*cond.clone())], int_params);
                    scan_for_ints(then_branch, int_params);
                    if let Some(e) = else_branch {
                        scan_for_ints(e, int_params);
                    }
                }

                Expr::While { cond, body } => {
                    scan_for_ints(&[Node::Expr(*cond.clone())], int_params);
                    scan_for_ints(body, int_params);
                }

                Expr::Funcy { body, .. } => {
                    scan_for_ints(body, int_params);
                }

                _ => {}
            },
            _ => {}
        }
    }
}


    scan_for_ints(body, &mut int_params);

    // 🧩 Define parameter types intelligently
    let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = params
        .iter()
        .map(|p| {
            if int_params.contains(p) {
                self.i32_type.into() // i32 for math variables
            } else {
                self.context.i8_type().ptr_type(AddressSpace::default()).into()
            }
        })
        .collect();

    // === Create the LLVM function ===
    let fn_type = self.i32_type.fn_type(&param_types, false);
    let function = if let Some(existing) = self.module.get_function(name) {
    existing
} else {
    self.module.add_function(name, fn_type, None)
};


    // === Entry block ===
    let entry = self.context.append_basic_block(function, "entry");
    let saved_block = self.builder.get_insert_block();
    self.builder.position_at_end(entry);

    // === Local scope ===
    let mut local_vars: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)> = HashMap::new();

    for (i, param_name) in params.iter().enumerate() {
        let param = function.get_nth_param(i as u32).unwrap();
        let param_val = param;
let is_int_param = int_params.contains(param_name);

let (alloca, ty): (PointerValue, BasicTypeEnum) = if is_int_param {
    let ty = self.i32_type.as_basic_type_enum();
    let alloca = self.builder.build_alloca(ty, param_name).unwrap();
    self.builder.build_store(alloca, param_val).unwrap();
    (alloca, ty)
} else {
    // Treat as string (i8*)
    let str_ty = self.context.i8_type().ptr_type(AddressSpace::default()).as_basic_type_enum();
    let alloca = self.builder.build_alloca(str_ty, param_name).unwrap();
    self.builder.build_store(alloca, param_val).unwrap();
    (alloca, str_ty)
};

local_vars.insert(param_name.clone(), (alloca, ty));

    }

    // === Replace vars ===
    let old_vars = std::mem::replace(&mut self.vars, local_vars);

    // === Compile body ===
    let mut last_val: Option<BasicValueEnum<'ctx>> = None;
    for node in body {
        last_val = self.compile_node(node);
    }

    // === Return handling ===
    let ret_val = match last_val {
        Some(BasicValueEnum::IntValue(iv)) => iv,
        _ => self.i32_type.const_int(0, false),
    };

    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
        self.builder.build_return(Some(&ret_val)).unwrap();
    }

    // === Restore ===
    self.vars = old_vars;
    if let Some(block) = saved_block {
        self.builder.position_at_end(block);
    }

    self.functions.insert(name.to_string(), function);
    function
}





    
}
impl<'ctx> Codegen<'ctx> {
    fn ensure_builder_position(&self) {
    if let Some(block) = self.builder.get_insert_block() {
        if block.get_terminator().is_some() {
            // Only reposition if the current block is part of an *active* function
            if let Some(func) = block.get_parent() {
                // Do NOT create multiple “fix_blocks” if already at an end block
                if func.get_last_basic_block() != Some(block) {
                    let fix_block = self.context.append_basic_block(func, "fix_block");
                    self.builder.position_at_end(fix_block);
                }
            }
        }
    }
}

}
impl<'ctx> Codegen<'ctx> {
    /// Safely emit a branch only if the current block has no terminator.
    fn safe_branch(&self, target: inkwell::basic_block::BasicBlock<'ctx>) {
        if let Some(block) = self.builder.get_insert_block() {
            if block.get_terminator().is_none() {
                self.builder.build_unconditional_branch(target).unwrap();
            }
        }
    }
}



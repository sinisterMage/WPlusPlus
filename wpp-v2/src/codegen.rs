use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::BasicTypeEnum,
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue},
    AddressSpace,
    OptimizationLevel,
};
use std::collections::HashMap;
use inkwell::types::BasicType;

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
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();
        let i32_type = context.i32_type();
        let exc_flag = context.bool_type().const_int(0, false);
let exc_value = context.i32_type().const_int(0, false);
let flag = module.add_global(context.bool_type(), None, "__exception_flag");
flag.set_initializer(&exc_flag);
let val = module.add_global(context.i32_type(), None, "__exception_value");
val.set_initializer(&exc_value);


        Self {
            context,
            module,
            builder,
            i32_type,
            vars: HashMap::new(),
            loop_stack: Vec::new(),
            switch_stack: Vec::new(),

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
        Expr::StringLiteral(s) => {
            let str_bytes = format!("{}\0", s);
            let gv = self.builder.build_global_string_ptr(&str_bytes, "strlit").unwrap();
            gv.as_pointer_value().into()
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
    let left_val = self.compile_expr(left.as_ref()).into_int_value();
    let right_val = self.compile_expr(right.as_ref()).into_int_value();

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
                // Declare printf if not yet defined
                let printf = self.module.get_function("printf").unwrap_or_else(|| {
                    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
                    let ty = self.context.i32_type().fn_type(&[i8ptr.into()], true);
                    self.module.add_function("printf", ty, None)
                });

                // Compile argument
                if args.is_empty() {
                    panic!("print() expects 1 argument");
                }
                let val = self.compile_expr(&args[0]);

                match val {
                    // === Print string ===
                    BasicValueEnum::PointerValue(pv) => {
                        let fmt = self.builder.build_global_string_ptr("%s\n", "fmt_s").unwrap();
                        let call_args: Vec<BasicMetadataValueEnum> =
                            vec![fmt.as_pointer_value().into(), pv.into()];
                        self.builder.build_call(printf, &call_args, "call_printf").unwrap();
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
                            let fmt = self.builder.build_global_string_ptr("%s", "fmt_bool").unwrap();
                            let true_args = vec![fmt.as_pointer_value().into(), true_str.as_pointer_value().into()];
                            self.builder.build_call(printf, &true_args, "printf_true").unwrap();
                            self.builder.build_unconditional_branch(merge_block).unwrap();

                            // ELSE block
                            self.builder.position_at_end(else_block);
                            let fmt = self.builder.build_global_string_ptr("%s", "fmt_bool").unwrap();
                            let false_args = vec![fmt.as_pointer_value().into(), false_str.as_pointer_value().into()];
                            self.builder.build_call(printf, &false_args, "printf_false").unwrap();
                            self.builder.build_unconditional_branch(merge_block).unwrap();

                            // MERGE block
                            self.builder.position_at_end(merge_block);
                        } else {
                            // Normal int print
                            let fmt = self.builder.build_global_string_ptr("%d\n", "fmt_d").unwrap();
                            let args = vec![fmt.as_pointer_value().into(), iv.into()];
                            self.builder.build_call(printf, &args, "call_printf").unwrap();
                        }
                    }

                    other => panic!("print(..): unsupported value type {:?}", other),
                }

                // Print returns dummy int (printf returns int)
                self.i32_type.const_int(0, false).into()
            } else {
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
    let val = self.compile_expr(expr).into_int_value();

    // set global flag/value
    let flag = self.module.get_global("__exception_flag").unwrap();
    let val_global = self.module.get_global("__exception_value").unwrap();
    self.builder.build_store(flag.as_pointer_value(), self.context.bool_type().const_int(1, false)).unwrap();
    self.builder.build_store(val_global.as_pointer_value(), val).unwrap();

    // jump to dummy end (simulate unwinding)
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let end_bb = self.context.append_basic_block(func, "throw_exit");
    self.builder.build_unconditional_branch(end_bb).unwrap();
    self.builder.position_at_end(end_bb);

    self.i32_type.const_int(0, false).into()
}

Expr::TryCatch { try_block, catch_var, catch_block } => {
    // backup initial exception flag
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let after_bb = self.context.append_basic_block(func, "try_after");
    let catch_bb = self.context.append_basic_block(func, "try_catch");

    // start try
    for node in try_block {
        self.compile_node(node);
    }

    // check flag
    let flag_ptr = self.module.get_global("__exception_flag").unwrap().as_pointer_value();
    let flag_val = self.builder.build_load(self.context.bool_type(), flag_ptr, "flag_load").unwrap();
    self.builder.build_conditional_branch(flag_val.into_int_value(), catch_bb, after_bb).unwrap();

    // catch block
    self.builder.position_at_end(catch_bb);
    if let Some(var_name) = catch_var {
        let val_ptr = self.module.get_global("__exception_value").unwrap().as_pointer_value();
        let val = self.builder.build_load(self.i32_type, val_ptr, "caught_val").unwrap();
        let local = self.builder.build_alloca(self.i32_type, var_name).unwrap();
        self.builder.build_store(local, val).unwrap();
        self.vars.insert(var_name.clone(), (local, self.i32_type.as_basic_type_enum()));
    }
    for node in catch_block {
        self.compile_node(node);
    }
    // reset flag
    self.builder.build_store(flag_ptr, self.context.bool_type().const_int(0, false)).unwrap();
    self.builder.build_unconditional_branch(after_bb).unwrap();

    // continue
    self.builder.position_at_end(after_bb);
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

        // Track last *int* expression (so main can return it). Otherwise return 0.
        let mut last_int: Option<IntValue> = None;

        for node in nodes {
            if let Some(v) = self.compile_node(node) {
                if let BasicValueEnum::IntValue(iv) = v {
                    last_int = Some(iv);
                }
            }
        }
        self.ensure_builder_position();
        let ret = last_int.unwrap_or_else(|| self.i32_type.const_int(0, false));
        let _ = self.builder.build_return(Some(&ret));

        function
    }

    pub fn run_jit(&self) {
    let engine = self.create_engine();

    unsafe {
        // === Cross-platform printf registration ===
        #[cfg(target_os = "linux")]
        {
            use libc::printf;
            if let Some(func) = self.module.get_function("printf") {
                engine.add_global_mapping(&func, printf as usize);
            }
        }

        #[cfg(target_os = "macos")]
        {
            use libc::printf;
            if let Some(func) = self.module.get_function("printf") {
                engine.add_global_mapping(&func, printf as usize);
            }
        }

        #[cfg(target_os = "windows")]
        {
            use libc::_printf;
            if let Some(func) = self.module.get_function("printf") {
                engine.add_global_mapping(&func, _printf as usize);
            }
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



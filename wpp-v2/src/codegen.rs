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
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();
        let i32_type = context.i32_type();

        Self {
            context,
            module,
            builder,
            i32_type,
            vars: HashMap::new(),
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
            let l = self.compile_expr(left);
            let r = self.compile_expr(right);

            match (l, r) {
                (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => match op.as_str() {
                    "+" => self.builder.build_int_add(li, ri, "addtmp").unwrap().into(),
                    "-" => self.builder.build_int_sub(li, ri, "subtmp").unwrap().into(),
                    "*" => self.builder.build_int_mul(li, ri, "multmp").unwrap().into(),
                    "/" => self.builder.build_int_signed_div(li, ri, "divtmp").unwrap().into(),
                    "==" => self.builder.build_int_compare(inkwell::IntPredicate::EQ, li, ri, "eqtmp").unwrap().into(),
                    "!=" => self.builder.build_int_compare(inkwell::IntPredicate::NE, li, ri, "netmp").unwrap().into(),
                    "<"  => self.builder.build_int_compare(inkwell::IntPredicate::SLT, li, ri, "lttmp").unwrap().into(),
                    ">"  => self.builder.build_int_compare(inkwell::IntPredicate::SGT, li, ri, "gttmp").unwrap().into(),
                    "<=" => self.builder.build_int_compare(inkwell::IntPredicate::SLE, li, ri, "letmp").unwrap().into(),
                    ">=" => self.builder.build_int_compare(inkwell::IntPredicate::SGE, li, ri, "getmp").unwrap().into(),
                    _ => panic!("Unsupported operator: {}", op),
                },
                _ => panic!("Binary operator '{}' requires integer operands", op),
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
            // === Compile condition ===
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
                                "if_cond",
                            )
                            .unwrap()
                    }
                }
                _ => panic!("Condition in if must be an integer or bool"),
            };

            // === Create blocks ===
            let parent_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            let then_bb = self.context.append_basic_block(parent_fn, "then");
            let else_bb = self.context.append_basic_block(parent_fn, "else");
            let merge_bb = self.context.append_basic_block(parent_fn, "merge");

            // Conditional branch
            self.builder.build_conditional_branch(cond_i1, then_bb, else_bb).unwrap();

            // THEN branch
            self.builder.position_at_end(then_bb);
            for node in then_branch {
                self.compile_node(node);
            }
            self.builder.build_unconditional_branch(merge_bb).unwrap();

            // ELSE branch
            self.builder.position_at_end(else_bb);
            if let Some(else_nodes) = else_branch {
                for node in else_nodes {
                    self.compile_node(node);
                }
            }
            self.builder.build_unconditional_branch(merge_bb).unwrap();

            // MERGE block
            self.builder.position_at_end(merge_bb);

            // Return dummy int 0 for if-statement
            self.i32_type.const_int(0, false).into()
        }

        // === Fallback ===
        _ => panic!("Unhandled expression: {:?}", expr),
    }
}

    /// Compile a statement. Returns last expression value (if any).
    pub fn compile_node(&mut self, node: &Node) -> Option<BasicValueEnum<'ctx>> {
        match node {
            Node::Let { name, value } => {
                let v = self.compile_expr(value);

                // create an alloca of the matching type
                let (alloca, ty): (PointerValue, BasicTypeEnum) = match v {
                    BasicValueEnum::IntValue(_) => {
                        let ty = self.i32_type.as_basic_type_enum();
                        let p = self.builder.build_alloca(ty, name).unwrap();
                        (p, ty)
                    }
                    BasicValueEnum::PointerValue(_) => {
                        let ty = self.context.i8_type().ptr_type(AddressSpace::default()).as_basic_type_enum();
                        let p = self.builder.build_alloca(ty, name).unwrap();
                        (p, ty)
                    }
                    other => panic!("let {} = <unsupported type: {:?}>", name, other),
                };

                let _ = self.builder.build_store(alloca, v);
                self.vars.insert(name.clone(), (alloca, ty));
                None
            }

            Node::Expr(expr) => Some(self.compile_expr(expr)),
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

        let ret = last_int.unwrap_or_else(|| self.i32_type.const_int(0, false));
        let _ = self.builder.build_return(Some(&ret));

        function
    }

    pub fn run_jit(&self) {
        let engine = self.create_engine();
        unsafe {
            let main_fn: unsafe extern "C" fn() -> i32 =
                std::mem::transmute(engine.get_function_address("main").unwrap());
            let result = main_fn();
            println!("✅ JIT result from main(): {}", result);
        }
    }
}

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
use std::ffi::CString;
use std::os::raw::c_char;
use inkwell::values::BasicValue;
use libc::malloc;


use crate::ast::{Expr, Node};
use crate::runtime;
use std::mem;
use std::ffi::c_void;
use inkwell::types::IntType;




#[derive(Clone)]
pub struct VarInfo<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub ty: BasicTypeEnum<'ctx>,
    pub is_const: bool,
}
/// The core LLVM code generator for W++
pub struct Codegen <'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub i32_type: inkwell::types::IntType<'ctx>,

    /// Symbol table: name -> (alloca ptr, type of the slot)
    pub vars: HashMap<String, VarInfo<'ctx>>,
        pub globals: HashMap<String, VarInfo<'ctx>>, // ✅ persistent globals
        pub loop_stack: Vec<(inkwell::basic_block::BasicBlock<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)>,
    pub switch_stack: Vec<inkwell::basic_block::BasicBlock<'ctx>>,
    exception_flag: Option<PointerValue<'ctx>>,
    exception_value_i32: Option<PointerValue<'ctx>>,
    exception_value_str: Option<PointerValue<'ctx>>,
        pub functions: HashMap<String, FunctionValue<'ctx>>,
}
fn get_or_declare_fn<'ctx>(
    module: &Module<'ctx>,
    context: &'ctx Context,
    name: &str,
    ret_ty: inkwell::types::BasicTypeEnum<'ctx>,
    args: &[BasicMetadataTypeEnum<'ctx>],
) -> FunctionValue<'ctx> {
    if let Some(existing) = module.get_function(name) {
        existing
    } else {
        let fn_type = ret_ty.fn_type(args, false);
        module.add_function(name, fn_type, None)
    }
}
impl<'ctx> Codegen<'ctx> {
    pub fn init_runtime_support(&self) {
        let void_ty = self.context.void_type();
        let i8_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
        let i32_ty = self.context.i32_type();

        // void wpp_print_value(void* ptr, int32_t type_id)
        let print_val_ty = void_ty.fn_type(&[i8_ptr_ty.into(), i32_ty.into()], false);
        self.module.add_function("wpp_print_value", print_val_ty, None);

        // void wpp_print_array(void* ptr)
        let arr_ty = void_ty.fn_type(&[i8_ptr_ty.into()], false);
        self.module.add_function("wpp_print_array", arr_ty, None);

        // void wpp_print_object(void* ptr)
        let obj_ty = void_ty.fn_type(&[i8_ptr_ty.into()], false);
        self.module.add_function("wpp_print_object", obj_ty, None);
    }
}


impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
    let module = context.create_module(name);
    let builder = context.create_builder();
    let i32_type = context.i32_type();

    // ✅ Create global vars for exception state
    let flag_global = module.add_global(context.bool_type(), None, "_wpp_exc_flag");
    flag_global.set_initializer(&context.bool_type().const_int(0, false));

    let exc_i32_global = module.add_global(context.i32_type(), None, "_wpp_exc_i32");
    exc_i32_global.set_initializer(&context.i32_type().const_int(0, false));

    let exc_str_global = module.add_global(context.i8_type().ptr_type(AddressSpace::default()), None, "_wpp_exc_str");
    exc_str_global.set_initializer(&context.i8_type().ptr_type(AddressSpace::default()).const_null());

    // ✅ Build struct once
    let codegen = Self {
        context,
        module,
        builder,
        i32_type,
        vars: HashMap::new(),
        loop_stack: Vec::new(),
        switch_stack: Vec::new(),
        exception_flag: Some(flag_global.as_pointer_value()),
        exception_value_i32: Some(exc_i32_global.as_pointer_value()),
        exception_value_str: Some(exc_str_global.as_pointer_value()),
        functions: HashMap::new(),
        globals: HashMap::new(),
    };

    // ✅ Register runtime externs right after initialization
    codegen.init_runtime_support();
    codegen.init_network_support(); // ✅ new

    // ✅ Return ready-to-use codegen
    codegen
}



pub fn init_network_support(&self) {
    use inkwell::AddressSpace;

    let void_ty = self.context.void_type();
    let i32_ty = self.context.i32_type();
    let i8_ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let fn_ptr = void_ty.fn_type(&[], false).ptr_type(AddressSpace::default());

    // === HTTP GET ===
    let http_get_ty = i32_ty.fn_type(&[i8_ptr.into()], false);
    self.module.add_function("wpp_http_get", http_get_ty, None);

    // === Register Endpoint ===
    let register_ty = void_ty.fn_type(&[i8_ptr.into(), fn_ptr.into()], false);
    self.module.add_function("wpp_register_endpoint", register_ty, None);

    // === Start Server ===
    let start_ty = void_ty.fn_type(&[i32_ty.into()], false);
    self.module.add_function("wpp_start_server", start_ty, None);
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
    let var = self.vars.get(name)
        .or_else(|| self.globals.get(name)) // ✅ fallback
        .unwrap_or_else(|| panic!("Unknown variable '{}'", name));

    self.builder
        .build_load(var.ty, var.ptr, &format!("load_{}", name))
        .unwrap()
        .into()
}

Expr::TypedLiteral { value, ty } => {
    match ty.as_str() {
        "i8"  => self.context.i8_type().const_int(value.parse::<i64>().unwrap() as u64, false).into(),
        "i32" => self.context.i32_type().const_int(value.parse::<i64>().unwrap() as u64, false).into(),
        "i64" => self.context.i64_type().const_int(value.parse::<i64>().unwrap() as u64, false).into(),
        "u64" => self.context.i64_type().const_int(value.parse::<u64>().unwrap(), false).into(),
        "f64" => self.context.f64_type().const_float(value.parse::<f64>().unwrap()).into(),
        _ => panic!("❌ Unknown literal type: {}", ty),
    }
}


        // === Binary operation ===
        Expr::BinaryOp { left, op, right } => {
    if op == "=" {
    println!("🧩 Detected assignment expression!");

    if let Expr::Variable(var_name) = left.as_ref() {
        println!("➡️ Assigning to variable: {}", var_name);
        let value = self.compile_expr(right.as_ref());

        // Lookup variable info (either local or global)
        if let Some(var) = self.vars.get(var_name).or_else(|| self.globals.get(var_name)) {
            println!("🔍 Found variable {} (is_const = {})", var_name, var.is_const);
            if var.is_const {
                panic!("❌ Cannot assign to constant variable '{}'", var_name);
            }

            let var_ty = var.ty;

            // ✅ Step 1: type-aware casting
            let casted_val: BasicValueEnum<'ctx> = match (value, var_ty) {
                // === Integer → Integer ===
                (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(int_ty)) => {
                    let rhs_bits = iv.get_type().get_bit_width();
                    let lhs_bits = int_ty.get_bit_width();

                    if rhs_bits == lhs_bits {
                        iv.as_basic_value_enum()
                    } else if rhs_bits < lhs_bits {
                        // smaller → larger
                        self.builder
                            .build_int_z_extend(iv, int_ty, "assign_zext")
                            .unwrap()
                            .as_basic_value_enum()
                    } else {
                        // larger → smaller
                        self.builder
                            .build_int_truncate(iv, int_ty, "assign_trunc")
                            .unwrap()
                            .as_basic_value_enum()
                    }
                }

                // === Float → Float ===
                (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(_)) => {
                    fv.as_basic_value_enum()
                }

                // === Int → Float ===
                (BasicValueEnum::IntValue(iv), BasicTypeEnum::FloatType(f_ty)) => {
                    self.builder
                        .build_signed_int_to_float(iv, f_ty, "assign_int2float")
                        .unwrap()
                        .as_basic_value_enum()
                }

                // === Float → Int ===
                (BasicValueEnum::FloatValue(fv), BasicTypeEnum::IntType(i_ty)) => {
                    self.builder
                        .build_float_to_signed_int(fv, i_ty, "assign_float2int")
                        .unwrap()
                        .as_basic_value_enum()
                }

                // === Pointer → Pointer ===
                (BasicValueEnum::PointerValue(pv), BasicTypeEnum::PointerType(_)) => {
                    pv.as_basic_value_enum()
                }

                // === Type mismatch ===
                (val, _) => panic!(
                    "❌ Type mismatch assigning {:?} to {:?}",
                    val.get_type(),
                    var_ty
                ),
            };

            // ✅ Step 2: Store the casted value
            self.builder.build_store(var.ptr, casted_val).unwrap();

            // ✅ Step 3: return i32(0) for consistency
            return self.i32_type.const_int(0, false).into();
        } else {
            panic!("Unknown variable in assignment: {}", var_name);
        }
    } else {
        panic!("Left-hand side of assignment must be a variable");
    }
}



    // === Arithmetic and comparison ===
    // === Arithmetic and comparison ===
let left_raw = self.compile_expr(left.as_ref());
let right_raw = self.compile_expr(right.as_ref());

let result: BasicValueEnum<'ctx> = match (left_raw, right_raw) {
    // --- Integer + Integer ---
    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
        let lhs_ty = l.get_type();
        let rhs_ty = r.get_type();

        // Auto-promote smaller integer → larger integer
        let (l_cast, r_cast, final_ty) = if lhs_ty.get_bit_width() == rhs_ty.get_bit_width() {
            (l, r, lhs_ty)
        } else if lhs_ty.get_bit_width() < rhs_ty.get_bit_width() {
            (
                self.builder.build_int_z_extend(l, rhs_ty, "l_promote").unwrap(),
                r,
                rhs_ty,
            )
        } else {
            (
                l,
                self.builder.build_int_z_extend(r, lhs_ty, "r_promote").unwrap(),
                lhs_ty,
            )
        };

        match op.as_str() {
            "+" => self.builder.build_int_add(l_cast, r_cast, "addtmp").unwrap().as_basic_value_enum(),
            "-" => self.builder.build_int_sub(l_cast, r_cast, "subtmp").unwrap().as_basic_value_enum(),
            "*" => self.builder.build_int_mul(l_cast, r_cast, "multmp").unwrap().as_basic_value_enum(),
            "/" => self.builder.build_int_signed_div(l_cast, r_cast, "divtmp").unwrap().as_basic_value_enum(),

            // Comparisons return i1 → no forced i32
            "==" => self.builder.build_int_compare(inkwell::IntPredicate::EQ, l_cast, r_cast, "eqtmp").unwrap().as_basic_value_enum(),
            "!=" => self.builder.build_int_compare(inkwell::IntPredicate::NE, l_cast, r_cast, "netmp").unwrap().as_basic_value_enum(),
            "<"  => self.builder.build_int_compare(inkwell::IntPredicate::SLT, l_cast, r_cast, "lttmp").unwrap().as_basic_value_enum(),
            "<=" => self.builder.build_int_compare(inkwell::IntPredicate::SLE, l_cast, r_cast, "letmp").unwrap().as_basic_value_enum(),
            ">"  => self.builder.build_int_compare(inkwell::IntPredicate::SGT, l_cast, r_cast, "gttmp").unwrap().as_basic_value_enum(),
            ">=" => self.builder.build_int_compare(inkwell::IntPredicate::SGE, l_cast, r_cast, "getmp").unwrap().as_basic_value_enum(),
            _ => panic!("Unsupported integer operator: {}", op),
        }
    }

    // --- Float + Float ---
    (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => match op.as_str() {
        "+" => self.builder.build_float_add(lf, rf, "fadd").unwrap().as_basic_value_enum(),
        "-" => self.builder.build_float_sub(lf, rf, "fsub").unwrap().as_basic_value_enum(),
        "*" => self.builder.build_float_mul(lf, rf, "fmul").unwrap().as_basic_value_enum(),
        "/" => self.builder.build_float_div(lf, rf, "fdiv").unwrap().as_basic_value_enum(),
        "==" => self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, lf, rf, "feq").unwrap().as_basic_value_enum(),
        "!=" => self.builder.build_float_compare(inkwell::FloatPredicate::ONE, lf, rf, "fne").unwrap().as_basic_value_enum(),
        "<"  => self.builder.build_float_compare(inkwell::FloatPredicate::OLT, lf, rf, "flt").unwrap().as_basic_value_enum(),
        "<=" => self.builder.build_float_compare(inkwell::FloatPredicate::OLE, lf, rf, "fle").unwrap().as_basic_value_enum(),
        ">"  => self.builder.build_float_compare(inkwell::FloatPredicate::OGT, lf, rf, "fgt").unwrap().as_basic_value_enum(),
        ">=" => self.builder.build_float_compare(inkwell::FloatPredicate::OGE, lf, rf, "fge").unwrap().as_basic_value_enum(),
        _ => panic!("Unsupported float operator: {}", op),
    },

    // --- Mixed: Int + Float ---
    (BasicValueEnum::IntValue(iv), BasicValueEnum::FloatValue(fv)) => {
        let f_ty = fv.get_type();
        let iv_cast = self.builder.build_signed_int_to_float(iv, f_ty, "int_to_float_lhs").unwrap();
        let new_expr = self.builder.build_float_add(iv_cast, fv, "promoted_add").unwrap();
        if ["+", "-", "*", "/"].contains(&op.as_str()) {
            new_expr.as_basic_value_enum()
        } else {
            panic!("Cannot compare mixed int/float types without explicit cast")
        }
    }

    (BasicValueEnum::FloatValue(fv), BasicValueEnum::IntValue(iv)) => {
        let f_ty = fv.get_type();
        let iv_cast = self.builder.build_signed_int_to_float(iv, f_ty, "int_to_float_rhs").unwrap();
        let new_expr = self.builder.build_float_add(fv, iv_cast, "promoted_add").unwrap();
        if ["+", "-", "*", "/"].contains(&op.as_str()) {
            new_expr.as_basic_value_enum()
        } else {
            panic!("Cannot compare mixed float/int types without explicit cast")
        }
    }

    _ => panic!("❌ Unsupported operand types for operator `{}`", op),
};

// ✅ Result is now type-accurate (no auto i32 cast)
result
        }





        // === Boolean literal ===
        Expr::BoolLiteral(value) => self.context.bool_type().const_int(*value as u64, false).into(),

        // === Function call (print, etc.) ===
        Expr::Call { name, args } => {
             if name == "print" {
    // === Declare runtime externs ===
    let void_ty = self.context.void_type();
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    let wpp_print_value = self.module.get_function("wpp_print_value").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i8ptr.into(), i32_ty.into()], false);
        self.module.add_function("wpp_print_value", ty, None)
    });

    // === Compile argument ===
    if args.is_empty() {
        panic!("print() expects at least one argument");
    }

    let val = self.compile_expr(&args[0]);

    // === Convert the value into a pointer to pass to C ===
    let casted_ptr = match val {
        BasicValueEnum::PointerValue(pv) => pv,
        BasicValueEnum::IntValue(iv) => {
            // Treat numbers as generic pointer (not dereferenced)
            self.builder
                .build_int_to_ptr(
                    iv,
                    i8ptr,
                    "int_as_ptr",
                )
                .unwrap()
        }
        BasicValueEnum::ArrayValue(av) => {
            // Store the array temporarily
            let tmp = self.builder.build_alloca(av.get_type(), "tmp_arr").unwrap();
            self.builder.build_store(tmp, av).unwrap();
            self.builder
                .build_pointer_cast(tmp, i8ptr, "casted_arr")
                .unwrap()
        }
        BasicValueEnum::StructValue(sv) => {
            // Store the struct temporarily
            let tmp = self.builder.build_alloca(sv.get_type(), "tmp_obj").unwrap();
            self.builder.build_store(tmp, sv).unwrap();
            self.builder
                .build_pointer_cast(tmp, i8ptr, "casted_obj")
                .unwrap()
        }
        _ => {
            println!("⚠️ Unsupported print type — using null");
            i8ptr.const_null()
        }
    };

    // === Infer runtime type_id ===
    // 1 = array, 2 = object, 0 = other
    let type_id = match &args[0] {
    Expr::Variable(name) => {
        if name == "arr" {
            i32_ty.const_int(1, false) // array
        } else if name == "obj" {
            i32_ty.const_int(2, false) // object
        } else {
            i32_ty.const_int(0, false)
        }
    }
    Expr::ArrayLiteral(_) => i32_ty.const_int(1, false),
    Expr::ObjectLiteral(_) => i32_ty.const_int(2, false),
    _ => i32_ty.const_int(0, false),
};

    // === Call the C runtime function ===
    self.builder
        .build_call(
            wpp_print_value,
            &[casted_ptr.into(), type_id.into()],
            "call_wpp_print_value",
        )
        .unwrap();

    // === Return dummy i32 ===
    return self.i32_type.const_int(0, false).into();
}
// === HTTP GET ===
else if name == "http.get" {
    // Expect one argument: a string literal or variable containing URL
    if args.len() != 1 {
        panic!("http.get() expects 1 argument (URL)");
    }

    let url_val = self.compile_expr(&args[0]);
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    // Ensure extern is declared
    let http_get_fn = self.module.get_function("wpp_http_get").unwrap_or_else(|| {
        let ty = i32_ty.fn_type(&[i8ptr.into()], false);
        self.module.add_function("wpp_http_get", ty, None)
    });

    // Call the extern function
    let call = self.builder
        .build_call(http_get_fn, &[url_val.into()], "call_http_get")
        .unwrap();

    return call.try_as_basic_value().left().unwrap_or_else(|| {
        i32_ty.const_int(0, false).into()
    });
}
else if name == "http.post" || name == "http.put" || name == "http.patch" || name == "http.delete" {
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    let (func_name, needs_body) = match name.as_str() {
        "http.post" => ("wpp_http_post", true),
        "http.put" => ("wpp_http_put", true),
        "http.patch" => ("wpp_http_patch", true),
        "http.delete" => ("wpp_http_delete", false),
        _ => unreachable!(),
    };

    // Validate argument count
    if (needs_body && args.len() != 2) || (!needs_body && args.len() != 1) {
        panic!("{} expects {} argument(s)", name, if needs_body { 2 } else { 1 });
    }

    // Compile URL
    let url_val = self.compile_expr(&args[0]);

    // Compile body if applicable
    let mut params = vec![url_val.into()];
    if needs_body {
        let body_val = self.compile_expr(&args[1]);
        params.push(body_val.into());
    }

    // Ensure extern is declared
    let fn_ty = if needs_body {
        i32_ty.fn_type(&[i8ptr.into(), i8ptr.into()], false)
    } else {
        i32_ty.fn_type(&[i8ptr.into()], false)
    };
    let extern_fn = self.module.get_function(func_name).unwrap_or_else(|| {
        self.module.add_function(func_name, fn_ty, None)
    });

    let call = self.builder
        .build_call(extern_fn, &params, &format!("call_{}", name))
        .unwrap();

    return call.try_as_basic_value().left().unwrap_or_else(|| {
        i32_ty.const_int(0, false).into()
    });
}

// === SERVER REGISTER ===
else if name == "server.register" {
    if args.len() != 2 {
        panic!("server.register() expects 2 arguments (path, handler)");
    }

    let path_val = self.compile_expr(&args[0]);
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let void_ty = self.context.void_type();

    let register_fn = self.module.get_function("wpp_register_endpoint").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i8ptr.into(), i8ptr.into()], false);
        self.module.add_function("wpp_register_endpoint", ty, None)
    });

    // Handler must be a function name (variable)
    let handler_name = if let Expr::Variable(ref s) = args[1] {
        s.clone()
    } else {
        panic!("Expected function name as second argument in server.register");
    };

    let handler_fn = self.functions.get(&handler_name)
        .unwrap_or_else(|| panic!("Unknown handler function `{}`", handler_name));

    self.builder
        .build_call(
            register_fn,
            &[
                path_val.into(),
                handler_fn.as_global_value().as_pointer_value().into(),
            ],
            "call_server_register",
        )
        .unwrap();

    return self.i32_type.const_int(0, false).into();
}

// === SERVER START ===
else if name == "server.start" {
    if args.len() != 1 {
        panic!("server.start() expects 1 argument (port)");
    }

    let port_val = self.compile_expr(&args[0]);
    let i32_ty = self.context.i32_type();
    let void_ty = self.context.void_type();

    let start_fn = self.module.get_function("wpp_start_server").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i32_ty.into()], false);
        self.module.add_function("wpp_start_server", ty, None)
    });

    // Call start server
    self.builder
        .build_call(start_fn, &[port_val.into()], "call_server_start")
        .unwrap();

    // 🕓 Add persistent wait
    let wait_fn = self.module.get_function("wpp_runtime_wait").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[], false);
        self.module.add_function("wpp_runtime_wait", ty, None)
    });

    self.builder
        .build_call(wait_fn, &[], "call_runtime_wait")
        .unwrap();

    return self.i32_type.const_int(0, false).into();
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
                if let Some(var) = self.vars.get(var_name) {
    if var.is_const {
        panic!("❌ Cannot assign to constant variable '{}'", var_name);
    }
    self.builder.build_store(var.ptr, value).unwrap();
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
    self.vars.insert(
    var.clone(),
    VarInfo {
        ptr: alloca,
        ty: str_ty.as_basic_type_enum(),
        is_const: false,
    },
);

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







Expr::Funcy { name, params, body, is_async } => {
    if *is_async {
        self.compile_async_funcy(name, params, body);
    } else {
        self.compile_funcy(name, params, body);
    }
    self.i32_type.const_int(0, false).into()
}




Expr::Return(expr_opt) => {
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let func_name = func.get_name().to_str().unwrap_or_default().to_string();

    // === Evaluate return expression (if any) ===
    let ret_val = if let Some(expr) = expr_opt {
        let v = self.compile_expr(expr);
        match v {
            BasicValueEnum::IntValue(iv) => iv,
            BasicValueEnum::PointerValue(pv) => {
                // Print string returns (for debugging)
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

    // === Async return signal ===
    // Only call wpp_return() if this is an async function (heuristic: name != "bootstrap_main" and present in module)
    let void_ty = self.context.void_type();
    if func_name != "bootstrap_main" {
        if let Some(f) = self.module.get_function("wpp_return") {
            self.builder
                .build_call(f, &[ret_val.into()], "async_return_signal")
                .unwrap();
        } else {
            let fn_ty = void_ty.fn_type(&[self.i32_type.into()], false);
            let f = self.module.add_function("wpp_return", fn_ty, None);
            self.builder
                .build_call(f, &[ret_val.into()], "async_return_signal")
                .unwrap();
        }
    }

    // === Build actual LLVM return ===
    self.builder.build_return(Some(&ret_val)).unwrap();

    // === Move builder to a fresh safe block ===
    let after_ret = self.context.append_basic_block(func, "after_return");
    self.builder.position_at_end(after_ret);

    self.i32_type.const_int(0, false).into()
}




Expr::Await(inner) => {
    // Compile the inner async call
    let _ = self.compile_expr(inner);

    // === Ensure wpp_yield() exists ===
    let void_ty = self.context.void_type();
    let yield_fn = self.module.get_function("wpp_yield").unwrap_or_else(|| {
        let fn_ty = void_ty.fn_type(&[], false);
        self.module.add_function("wpp_yield", fn_ty, None)
    });

    // Yield control to scheduler
    self.builder.build_call(yield_fn, &[], "await_yield").unwrap();

    // === Ensure wpp_get_last_result() exists ===
    let get_last_result_fn = self.module.get_function("wpp_get_last_result").unwrap_or_else(|| {
        let fn_ty = self.i32_type.fn_type(&[], false);
        self.module.add_function("wpp_get_last_result", fn_ty, None)
    });

    // Load last async return value
    let call_res = self
        .builder
        .build_call(get_last_result_fn, &[], "await_result")
        .unwrap();

    // ✅ Unwrap the Option<BasicValueEnum>
    call_res
        .try_as_basic_value()
        .left()
        .expect("wpp_get_last_result() must return a value")
}


Expr::ArrayLiteral(elements) => {
    let element_count = elements.len() as u64;
    let i32_type = self.context.i32_type();
    let i64_type = self.context.i64_type();
    let elem_ty = i32_type; // default element type for now

    // === Allocate memory: (len + elements)
    // Total slots = element_count + 1 (store length first)
    let total_slots = element_count + 1;

    // Convert total_slots -> i64
    let total_slots_val = i64_type.const_int(total_slots, false);

    // size_of(i32) as i64
   // === Compute sizeof(i32) as i64 ===
let one = i32_type.const_int(1, false);
let gep = unsafe {
    self.builder.build_gep(
        i32_type,
        i32_type.ptr_type(AddressSpace::default()).const_null(),
        &[one],
        "size_of_i32",
    ).unwrap()
};

let i32_size_i64 = self.builder.build_ptr_to_int(
    gep,
    i64_type,
    "i32_size_i64",
).unwrap();


    // total bytes = sizeof(i32) * total_slots
    let total_size_bytes = self.builder.build_int_mul(i32_size_i64, total_slots_val, "arr_total_bytes");

    // malloc(i64)
    let malloc_ty = self.context.i8_type().ptr_type(AddressSpace::default());
    let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
        let ty = malloc_ty.fn_type(&[i64_type.into()], false);
        self.module.add_function("malloc", ty, None)
    });

    let total_size_bytes_val = total_size_bytes.unwrap();
let mem_ptr_i8 = self.builder
    .build_call(
        malloc_fn,
        &[total_size_bytes_val.into()],
        "arr_malloc",
    )
    .unwrap()
    .try_as_basic_value()
    .left()
    .expect("malloc must return pointer")
    .into_pointer_value();


    // Cast to i32*
    let arr_ptr = self.builder
        .build_bitcast(mem_ptr_i8, i32_type.ptr_type(AddressSpace::default()), "arr_cast")
        .unwrap()
        .into_pointer_value();

    // === Store length at [0]
    self.builder.build_store(arr_ptr, i32_type.const_int(element_count, false)).unwrap();

    // === Compile and store each element sequentially
    for (i, el) in elements.iter().enumerate() {
        let val = self.compile_expr(el);
        let offset = i32_type.const_int((i + 1) as u64, false);
        let elem_ptr = unsafe {
            self.builder.build_gep(i32_type, arr_ptr, &[offset], "elem_ptr").unwrap()
        };

        match val {
            BasicValueEnum::IntValue(iv) => {
                self.builder.build_store(elem_ptr, iv).unwrap();
            }
            BasicValueEnum::FloatValue(fv) => {
                let iv = self.builder.build_float_to_signed_int(fv, i32_type, "arr_cast_float").unwrap();
                self.builder.build_store(elem_ptr, iv).unwrap();
            }
            BasicValueEnum::PointerValue(_) => {
                println!("⚠️ Pointer elements not yet supported in array literal");
            }
            _ => {}
        }
    }

    arr_ptr.as_basic_value_enum()
}

Expr::ObjectLiteral(fields) => {
    let field_count = fields.len() as u64;
    let i32_type = self.context.i32_type();
    let i64_type = self.context.i64_type();
    let i8_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());

    // === Define runtime struct: { i32 field_count, i8** keys, i32* values }
    let struct_ty = self.context.struct_type(
        &[
            i32_type.into(),
            i8_ptr_ty.ptr_type(AddressSpace::default()).into(),
            i32_type.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );

    // === malloc(i64)
    let malloc_ty = self.context.i8_type().ptr_type(AddressSpace::default());
    let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
        let ty = malloc_ty.fn_type(&[i64_type.into()], false);
        self.module.add_function("malloc", ty, None)
    });

    // === Compute sizeof(struct_ty)
    let one = i32_type.const_int(1, false);
    let gep = unsafe {
        self.builder
            .build_gep(
                struct_ty,
                struct_ty.ptr_type(AddressSpace::default()).const_null(),
                &[one],
                "sizeof_struct",
            )
            .unwrap()
    };

    let struct_size_i64 = self
        .builder
        .build_ptr_to_int(gep, i64_type, "struct_size_i64")
        .unwrap();

    // === Allocate struct
    let mem_ptr_i8 = self
        .builder
        .build_call(malloc_fn, &[struct_size_i64.into()], "obj_malloc")
        .unwrap()
        .try_as_basic_value()
        .left()
        .expect("malloc must return pointer")
        .into_pointer_value();

    let obj_ptr = self
        .builder
        .build_bitcast(
            mem_ptr_i8,
            struct_ty.ptr_type(AddressSpace::default()),
            "obj_cast",
        )
        .unwrap()
        .into_pointer_value();

    // === Compute sizeof(i32) as i64
    let one = i32_type.const_int(1, false);
    let gep = unsafe {
        self.builder
            .build_gep(
                i32_type,
                i32_type.ptr_type(AddressSpace::default()).const_null(),
                &[one],
                "sizeof_i32",
            )
            .unwrap()
    };
    let i32_size_i64 = self
        .builder
        .build_ptr_to_int(gep, i64_type, "i32_size_i64")
        .unwrap();

    // === Compute total bytes for arrays
    let field_count_val = i64_type.const_int(field_count, false);
    let total_keys_bytes = self
        .builder
        .build_int_mul(i32_size_i64, field_count_val, "total_keys_bytes")
        .unwrap();
    let total_vals_bytes = self
        .builder
        .build_int_mul(i32_size_i64, field_count_val, "total_vals_bytes")
        .unwrap();

    // === Allocate keys array
    let keys_mem_i8 = self
        .builder
        .build_call(malloc_fn, &[total_keys_bytes.into()], "keys_malloc")
        .unwrap()
        .try_as_basic_value()
        .left()
        .expect("malloc must return pointer")
        .into_pointer_value();

    let keys_ptr = self
        .builder
        .build_bitcast(
            keys_mem_i8,
            i8_ptr_ty.ptr_type(AddressSpace::default()),
            "keys_cast",
        )
        .unwrap()
        .into_pointer_value();

    // === Allocate values array
    let vals_mem_i8 = self
        .builder
        .build_call(malloc_fn, &[total_vals_bytes.into()], "vals_malloc")
        .unwrap()
        .try_as_basic_value()
        .left()
        .expect("malloc must return pointer")
        .into_pointer_value();

    let vals_ptr = self
        .builder
        .build_bitcast(
            vals_mem_i8,
            i32_type.ptr_type(AddressSpace::default()),
            "vals_cast",
        )
        .unwrap()
        .into_pointer_value();

    // === Populate keys + values
    for (i, (key, val)) in fields.iter().enumerate() {
        let val_compiled = self.compile_expr(val);

        // Store key as constant string
        let key_const = self.context.const_string(key.as_bytes(), true);
        let gname = format!("objkey_{}", i);
        let gkey = self.module.add_global(key_const.get_type(), None, &gname);
        gkey.set_initializer(&key_const);
        gkey.set_constant(true);
        gkey.set_linkage(Linkage::Private);
        let key_ptr = gkey.as_pointer_value();

        let key_index = i32_type.const_int(i as u64, false);
        let key_slot = unsafe {
            self.builder
                .build_gep(i8_ptr_ty, keys_ptr, &[key_index], "key_slot")
                .unwrap()
        };
        self.builder.build_store(key_slot, key_ptr).unwrap();

        // Store value (convert to i32 if needed)
        let val_i32 = match val_compiled {
            BasicValueEnum::IntValue(iv) => iv,
            BasicValueEnum::FloatValue(fv) => self
                .builder
                .build_float_to_signed_int(fv, i32_type, "val_float2int")
                .unwrap(),
            _ => i32_type.const_int(0, false),
        };
        let val_slot = unsafe {
            self.builder
                .build_gep(i32_type, vals_ptr, &[key_index], "val_slot")
                .unwrap()
        };
        self.builder.build_store(val_slot, val_i32).unwrap();
    }

    // === Fill struct fields
    let field_0 =
        unsafe { self.builder.build_struct_gep(struct_ty, obj_ptr, 0, "f0").unwrap() };
    self.builder
        .build_store(field_0, i32_type.const_int(field_count, false))
        .unwrap();

    let field_1 =
        unsafe { self.builder.build_struct_gep(struct_ty, obj_ptr, 1, "f1").unwrap() };
    self.builder.build_store(field_1, keys_ptr).unwrap();

    let field_2 =
        unsafe { self.builder.build_struct_gep(struct_ty, obj_ptr, 2, "f2").unwrap() };
    self.builder.build_store(field_2, vals_ptr).unwrap();

    obj_ptr.as_basic_value_enum()
}





        // === Fallback ===
        _ => panic!("Unhandled expression: {:?}", expr),
    }
}
fn resolve_basic_type(&self, ty: &str) -> inkwell::types::BasicTypeEnum<'ctx> {
    match ty {
        "i8"  => self.context.i8_type().into(),
        "i32" => self.context.i32_type().into(),
        "i64" => self.context.i64_type().into(),
        "u64" => self.context.i64_type().into(), // LLVM doesn’t distinguish signed vs unsigned types
        "f64" => self.context.f64_type().into(),
        _ => panic!("❌ Unknown type: {}", ty),
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
        
        Node::Let { name, value, is_const, ty } => {
    println!("🧱 Compiling top-level node: Let {{ name: {}, ty: {:?} }}", name, ty);

    // === Detect heap-allocated expressions (arrays/objects) ===
    let is_heap_value = matches!(value, Expr::ArrayLiteral(_) | Expr::ObjectLiteral(_));
    if is_heap_value {
        println!("💾 Variable `{}` is a heap object — allocating as pointer", name);
    }

    // === Determine variable type ===
    let var_type: BasicTypeEnum<'ctx> = if is_heap_value {
        // All heap structures are stored as pointers (i8*)
        self.context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .as_basic_type_enum()
    } else if let Some(t) = ty {
        // Explicit type annotation
        self.resolve_basic_type(t)
    } else {
        // 🔍 Type inference from RHS
        match value {
            Expr::TypedLiteral { value: val, ty: lit_ty } => match lit_ty.as_str() {
                "i1" | "bool" => self.context.bool_type().into(),
                "i8" => self.context.i8_type().into(),
                "i32" => self.context.i32_type().into(),
                "i64" => self.context.i64_type().into(),
                "f64" => self.context.f64_type().into(),
                _ => self.infer_type_from_literal(val, lit_ty == "f64"),
            },

            Expr::Literal(v) => self.infer_type_from_literal(&v.to_string(), false),

            Expr::BoolLiteral(_) => self.context.bool_type().into(),

            Expr::Variable(var_name) => {
                if let Some(existing) = self.vars.get(var_name).or_else(|| self.globals.get(var_name)) {
                    existing.ty
                } else {
                    println!("⚠️ Unknown variable `{}`, defaulting to i32", var_name);
                    self.i32_type.into()
                }
            }

            Expr::BinaryOp { left, right, .. } => {
                let left_ty = self.infer_expr_type(left);
                let right_ty = self.infer_expr_type(right);
                self.merge_types(left_ty, right_ty)
            }

            _ => {
                println!("⚠️ Complex expression — defaulting to i32");
                self.i32_type.into()
            }
        }
    };

    // === Allocate space for variable ===
    let alloca = self.builder.build_alloca(var_type, name).unwrap();

    // === Compile RHS ===
    let rhs_val = self.compile_expr(value);

    // === Store safely ===
    if is_heap_value {
        // Heap values already return a pointer; store directly
        self.builder.build_store(alloca, rhs_val).unwrap();
    } else {
        // Primitive values: perform safe casting if needed
        let casted_val = match (rhs_val, var_type) {
            (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(int_ty)) => {
                let rhs_bits = iv.get_type().get_bit_width();
                let lhs_bits = int_ty.get_bit_width();
                if rhs_bits == lhs_bits {
                    iv.as_basic_value_enum()
                } else if rhs_bits < lhs_bits {
                    self.builder
                        .build_int_z_extend(iv, int_ty, "zext")
                        .unwrap()
                        .as_basic_value_enum()
                } else {
                    self.builder
                        .build_int_truncate(iv, int_ty, "trunc")
                        .unwrap()
                        .as_basic_value_enum()
                }
            }

            (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(_)) => fv.as_basic_value_enum(),

            (BasicValueEnum::IntValue(iv), BasicTypeEnum::FloatType(f_ty)) => {
                self.builder
                    .build_signed_int_to_float(iv, f_ty, "int2float")
                    .unwrap()
                    .as_basic_value_enum()
            }

            (BasicValueEnum::FloatValue(fv), BasicTypeEnum::IntType(i_ty)) => {
                self.builder
                    .build_float_to_signed_int(fv, i_ty, "float2int")
                    .unwrap()
                    .as_basic_value_enum()
            }

            (BasicValueEnum::IntValue(iv), BasicTypeEnum::PointerType(_)) => iv.as_basic_value_enum(),

            _ => rhs_val,
        };

        self.builder.build_store(alloca, casted_val).unwrap();
    }

    // === Register variable ===
    self.vars.insert(
        name.clone(),
        VarInfo {
            ptr: alloca,
            ty: var_type,
            is_const: *is_const,
        },
    );

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
    /// Create main(), compile nodes, and return i32.

/// Create main(), compile nodes, and return i32.
pub fn compile_main(&mut self, nodes: &[Node]) -> FunctionValue<'ctx> {
    let fn_type = self.i32_type.fn_type(&[], false);
    let function_name = "main_async";
    let function = self.module.add_function(function_name, fn_type, None);
let entry = self.context.append_basic_block(function, "entry");
self.builder.position_at_end(entry);

// Keep handle for later verification
let main_entry_block = entry;

    // === Exception slots ===
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

    // === Pass 1: predeclare all function signatures ===
    for node in nodes {
        if let Node::Expr(Expr::Funcy { name, params, .. }) = node {
            let param_types: Vec<_> = params.iter().map(|_| self.i32_type.into()).collect();
            let fn_type = self.i32_type.fn_type(&param_types, false);
            let f = self.module.add_function(name, fn_type, None);
            self.functions.insert(name.clone(), f);
        }
    }

    // === Pass 2: compile all function bodies ===
    for node in nodes {
        if let Node::Expr(Expr::Funcy { name, params, body, is_async }) = node {
            if *is_async {
                self.compile_async_funcy(name, params, body);
            } else {
                self.compile_funcy(name, params, body);
            }
        }
    }
// === Ensure main_async has a proper terminator ===
// (Do NOT insert return yet — we’ll do it after top-level compilation)

// === Detect async entry (prefer async funcy main) ===
let async_entry = nodes.iter().find_map(|n| {
    if let Node::Expr(Expr::Funcy { name, is_async: true, .. }) = n {
        if name == "main" { Some(name.clone()) } else { None }
    } else { None }
}).or_else(|| {
    nodes.iter().find_map(|n| {
        if let Node::Expr(Expr::Funcy { name, is_async: true, .. }) = n {
            Some(name.clone())
        } else { None }
    })
});

// === Async bootstrap ===
if let Some(ref entry_name) = async_entry {
    if let Some(fn_val) = self.module.get_function(entry_name) {
        println!("⚡ Injecting async entry `{}` via bootstrap", entry_name);

        let fn_type = self.i32_type.fn_type(&[], false);
        let bootstrap = self.module.add_function("main", fn_type, None);
        let entry_block = self.context.append_basic_block(bootstrap, "entry");
        self.builder.position_at_end(entry_block);

        // externs
        let void_ty = self.context.void_type();
        let fn_ptr_ty = void_ty.fn_type(&[], false).ptr_type(AddressSpace::default());
        let spawn_ty = void_ty.fn_type(&[fn_ptr_ty.into()], false);
        let yield_ty = void_ty.fn_type(&[], false);

        let spawn_fn = self.module.get_function("wpp_spawn").unwrap_or_else(|| {
            self.module.add_function("wpp_spawn", spawn_ty, None)
        });
        let yield_fn = self.module.get_function("wpp_yield").unwrap_or_else(|| {
            self.module.add_function("wpp_yield", yield_ty, None)
        });

        // spawn and start scheduler
        self.builder
            .build_call(
                spawn_fn,
                &[fn_val.as_global_value().as_pointer_value().into()],
                "spawn_main_task",
            )
            .unwrap();
        self.builder.build_call(yield_fn, &[], "start_scheduler").unwrap();

        let ret_val = self.i32_type.const_int(0, false);
        self.builder.build_return(Some(&ret_val)).unwrap();

        // move builder safely again
        let safe_block = self.context.append_basic_block(bootstrap, "after_return");
        self.builder.position_at_end(safe_block);

        return bootstrap;
    }
}

// === No async entry: compile top-level ===
let mut last_int: Option<IntValue> = None;
for node in nodes {
    println!("🧱 Compiling top-level node: {:?}", node);
    if let Some(v) = self.compile_node(node) {
        if let BasicValueEnum::IntValue(iv) = v {
            last_int = Some(iv);
        }
    }
}

// === Ensure main() exists BEFORE returning ===
if self.module.get_function("main").is_none() {
    if let Some(main_async) = self.module.get_function("main_async") {
        println!("🔗 Generating wrapper main() -> main_async");

        let fn_type = self.i32_type.fn_type(&[], false);
        let wrapper = self.module.add_function("main", fn_type, None);
        let entry = self.context.append_basic_block(wrapper, "entry");
        self.builder.position_at_end(entry);

        let call_site = self.builder.build_call(main_async, &[], "call_main_async").unwrap();
        let result = call_site
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| self.i32_type.const_int(0, false).into());

        self.builder.build_return(Some(&result.into_int_value())).unwrap();

        // move builder safely
        let safe_block = self.context.append_basic_block(wrapper, "after_return");
        self.builder.position_at_end(safe_block);
    }
}

// === Final top-level return (if no async entry) ===
if async_entry.is_none() {
    let ret_val = last_int.unwrap_or_else(|| self.i32_type.const_int(0, false));
    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
        self.builder.build_return(Some(&ret_val)).unwrap();
    }
}

// ✅ Ensure the main function ends with a valid return instruction
// === FINAL SAFETY: Ensure main_async's entry block is terminated ===
if main_entry_block.get_terminator().is_none() {
    let temp_builder = self.context.create_builder();
    temp_builder.position_at_end(main_entry_block);
    let ret_val = self.i32_type.const_int(0, false);
    temp_builder.build_return(Some(&ret_val)).unwrap();
    println!("🟢 Added final return terminator to main_async::entry");
}



function

}








   pub fn run_jit(&self) {
    use std::mem;
    use libc::printf;
    use crate::runtime;

    let engine: ExecutionEngine<'_> = self.create_engine();

    unsafe {
        crate::runtime::set_engine(mem::transmute::<
            ExecutionEngine<'_>,
            ExecutionEngine<'static>,
        >(engine.clone()));
    }
    println!("✅ [jit] Execution engine registered globally");

    unsafe {
        // printf (system)
        if let Some(func) = self.module.get_function("printf") {
            engine.add_global_mapping(&func, printf as usize);
        }

        // === Runtime: spawn / yield / return ===
        extern "C" fn wpp_spawn_stub(ptr: *const std::ffi::c_void) {
            runtime::wpp_spawn(ptr as *const ());
        }

        extern "C" fn wpp_yield_stub() {
            runtime::wpp_yield();
        }

        extern "C" fn wpp_return_stub(val: i32) {
            runtime::wpp_return(val);
        }

        extern "C" fn wpp_get_last_result_stub() -> i32 {
            runtime::wpp_get_last_result()
        }

        if let Some(spawn) = self.module.get_function("wpp_spawn") {
            engine.add_global_mapping(&spawn, wpp_spawn_stub as usize);
        }
        
    unsafe extern "C" {
        fn wpp_http_get(ptr: *const std::os::raw::c_char) -> i32;
        fn wpp_register_endpoint(path: *const std::os::raw::c_char, handler: *const ());
        fn wpp_start_server(port: i32);
    }

    if let Some(func) = self.module.get_function("wpp_http_get") {
        engine.add_global_mapping(&func, wpp_http_get as usize);
    }
    if let Some(func) = self.module.get_function("wpp_register_endpoint") {
        engine.add_global_mapping(&func, wpp_register_endpoint as usize);
    }
    if let Some(func) = self.module.get_function("wpp_start_server") {
        engine.add_global_mapping(&func, wpp_start_server as usize);
    }


        if let Some(yield_fn) = self.module.get_function("wpp_yield") {
            engine.add_global_mapping(&yield_fn, wpp_yield_stub as usize);
        }

        if let Some(ret_fn) = self.module.get_function("wpp_return") {
            engine.add_global_mapping(&ret_fn, wpp_return_stub as usize);
        }

        if let Some(get_last_fn) = self.module.get_function("wpp_get_last_result") {
            engine.add_global_mapping(&get_last_fn, wpp_get_last_result_stub as usize);
        }

        if let Some(func) = self.module.get_function("malloc") {
            engine.add_global_mapping(&func, libc::malloc as usize);
        }

        // === 🔗 NEW: connect print runtime ===
        // === 🔗 NEW: connect print runtime ===
unsafe extern "C" {
    fn wpp_print_value(ptr: *const std::ffi::c_void, type_id: i32);
    fn wpp_print_array(ptr: *const std::ffi::c_void);
    fn wpp_print_object(ptr: *const std::ffi::c_void);
}

unsafe extern "C" {
    fn wpp_http_post(url: *const std::os::raw::c_char, body: *const std::os::raw::c_char) -> i32;
    fn wpp_http_put(url: *const std::os::raw::c_char, body: *const std::os::raw::c_char) -> i32;
    fn wpp_http_patch(url: *const std::os::raw::c_char, body: *const std::os::raw::c_char) -> i32;
    fn wpp_http_delete(url: *const std::os::raw::c_char) -> i32;
}

if let Some(func) = self.module.get_function("wpp_http_post") {
    engine.add_global_mapping(&func, wpp_http_post as usize);
}
if let Some(func) = self.module.get_function("wpp_http_put") {
    engine.add_global_mapping(&func, wpp_http_put as usize);
}
if let Some(func) = self.module.get_function("wpp_http_patch") {
    engine.add_global_mapping(&func, wpp_http_patch as usize);
}
if let Some(func) = self.module.get_function("wpp_http_delete") {
    engine.add_global_mapping(&func, wpp_http_delete as usize);
}


        if let Some(func) = self.module.get_function("wpp_print_value") {
            engine.add_global_mapping(&func, wpp_print_value as usize);
        }
        if let Some(func) = self.module.get_function("wpp_print_array") {
            engine.add_global_mapping(&func, wpp_print_array as usize);
        }
        if let Some(func) = self.module.get_function("wpp_print_object") {
            engine.add_global_mapping(&func, wpp_print_object as usize);
        }
        unsafe extern "C" {
    fn wpp_runtime_wait();
}

if let Some(func) = self.module.get_function("wpp_runtime_wait") {
    engine.add_global_mapping(&func, wpp_runtime_wait as usize);
}

    }

    // ✅ Launch entrypoint
    let entry_name = if self.module.get_function("bootstrap_main").is_some() {
        "bootstrap_main"
    } else {
        "main"
    };

    println!("🚀 [jit] Launching entrypoint: {}", entry_name);

    let entry_addr = engine
        .get_function_address(entry_name)
        .unwrap_or_else(|_| panic!("❌ Could not find function `{}` in module", entry_name));

    let entry_fn: extern "C" fn() -> i32 = unsafe { mem::transmute(entry_addr) };
    let result = entry_fn();

    println!("🏁 [jit] Finished running {}, result = {}", entry_name, result);
}


fn infer_type_from_literal(&self, raw: &str, is_float: bool) -> BasicTypeEnum<'ctx> {
    if is_float || raw.contains('.') {
        self.context.f64_type().into()
    } else {
        match raw.parse::<i64>() {
            Ok(v) if v >= i8::MIN as i64 && v <= i8::MAX as i64 => self.context.i8_type().into(),
            Ok(v) if v >= i32::MIN as i64 && v <= i32::MAX as i64 => self.context.i32_type().into(),
            Ok(_) => self.context.i64_type().into(),
            Err(_) => panic!("❌ Invalid numeric literal: {}", raw),
        }
    }
}

fn infer_expr_type(&self, expr: &Expr) -> BasicTypeEnum<'ctx> {
    match expr {
        Expr::TypedLiteral { ty, .. } => self.resolve_basic_type(ty),
        Expr::Literal(v) => self.infer_type_from_literal(&v.to_string(), false),
        Expr::BoolLiteral(_) => self.context.bool_type().into(),
        Expr::Variable(name) => {
            self.vars
                .get(name)
                .or_else(|| self.globals.get(name))
                .map(|v| v.ty)
                .unwrap_or_else(|| self.i32_type.into())
        }
        Expr::BinaryOp { left, right, .. } => {
            let l = self.infer_expr_type(left);
            let r = self.infer_expr_type(right);
            self.merge_types(l, r)
        }
        _ => self.i32_type.into(),
    }
}

fn merge_types(&self, left: BasicTypeEnum<'ctx>, right: BasicTypeEnum<'ctx>) -> BasicTypeEnum<'ctx> {
    if left == right {
        return left;
    }

    // f64 dominance for mixed math
    if left.is_float_type() || right.is_float_type() {
        return self.context.f64_type().into();
    }

    // Choose widest integer type
    let l_bits = if let BasicTypeEnum::IntType(t) = left { t.get_bit_width() } else { 0 };
    let r_bits = if let BasicTypeEnum::IntType(t) = right { t.get_bit_width() } else { 0 };
    let max_bits = std::cmp::max(l_bits, r_bits);

    match max_bits {
        1 => self.context.bool_type().into(),
        8 => self.context.i8_type().into(),
        32 => self.context.i32_type().into(),
        64 => self.context.i64_type().into(),
        _ => self.i32_type.into(),
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
let mut local_vars: HashMap<String, VarInfo<'ctx>> = HashMap::new();

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

local_vars.insert(
    param_name.clone(),
    VarInfo {
        ptr: alloca,
        ty,
        is_const: false,
    },
);


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

pub fn compile_async_funcy(
    &mut self,
    name: &str,
    params: &[String],
    body: &[Node],
) -> FunctionValue<'ctx> {
    // === Remove stale definitions (for hot recompilation) ===
    if let Some(existing) = self.module.get_function(name) {
        unsafe { let _ = existing.delete(); }
    }

    println!("⚙️ compiling async funcy {}", name);

    // === Define function signature (i32 return, i32 params) ===
    let param_types: Vec<BasicMetadataTypeEnum<'ctx>> =
        params.iter().map(|_| self.i32_type.into()).collect();
    let fn_type = self.i32_type.fn_type(&param_types, false);

    // === Create function + entry block ===
    let function = self.module.add_function(name, fn_type, None);
    let entry = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(entry);

    // === Allocate and store parameters ===
let mut local_vars: HashMap<String, VarInfo<'ctx>> = HashMap::new();
    for (i, pname) in params.iter().enumerate() {
        let param = function.get_nth_param(i as u32).unwrap();
        let alloca = self.builder.build_alloca(self.i32_type, pname).unwrap();
        self.builder.build_store(alloca, param).unwrap();
        local_vars.insert(
    pname.clone(),
    VarInfo {
        ptr: alloca,
        ty: self.i32_type.as_basic_type_enum(),
        is_const: false,
    },
);

    }

    // === Scoped variable map ===
    let old_vars: HashMap<String, VarInfo<'ctx>> = std::mem::replace(&mut self.vars, local_vars);

    // === Compile body ===
    let mut last_val: Option<BasicValueEnum<'ctx>> = None;
    for node in body {
        last_val = self.compile_node(node);
    }

    // === Determine return value (default to 0) ===
    let ret_val = match last_val {
        Some(BasicValueEnum::IntValue(iv)) => iv,
        _ => self.i32_type.const_int(0, false),
    };

    // === Add async footer only if block isn’t already terminated ===
    if self
        .builder
        .get_insert_block()
        .and_then(|b| b.get_terminator())
        .is_none()
    {
        let void_ty = self.context.void_type();

        // wpp_return(i32)
        let wpp_return = self.module.get_function("wpp_return").unwrap_or_else(|| {
            let fn_ty = void_ty.fn_type(&[self.i32_type.into()], false);
            self.module.add_function("wpp_return", fn_ty, None)
        });

        // wpp_yield()
        let yield_fn = self.module.get_function("wpp_yield").unwrap_or_else(|| {
            let fn_ty = void_ty.fn_type(&[], false);
            self.module.add_function("wpp_yield", fn_ty, None)
        });

        // --- async return footer ---
        // 1️⃣ Notify runtime of result (only once)
        self.builder
            .build_call(wpp_return, &[ret_val.into()], "final_async_return")
            .unwrap();

        // 2️⃣ Yield to scheduler to finalize
        self.builder.build_call(yield_fn, &[], "yield_after_return").unwrap();

        // 3️⃣ Actually return from LLVM func
        self.builder.build_return(Some(&ret_val)).unwrap();
    } else {
        println!("⚠️ [compile_async_funcy] Block already terminated, skipping footer");
    }

    // === Restore outer variable scope ===
    self.vars = old_vars;
    self.functions.insert(name.to_string(), function);

    println!("✅ async funcy {} compiled successfully", name);
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



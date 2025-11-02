use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::{self, Linkage, Module},
    types::BasicTypeEnum,
    values::{AnyValue, BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue, IntValue, PointerValue},
    AddressSpace,
    OptimizationLevel,
};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use inkwell::types::BasicType;
use inkwell::types::BasicMetadataTypeEnum;
use std::ffi::CString;
use std::os::raw::c_char;
use inkwell::values::BasicValue;
use libc::malloc;


use crate::ast::{node::{EntityMember, EntityNode}, Expr, Node};
use crate::ast::types::TypeDescriptor;
use crate::runtime;
use std::mem;
use std::ffi::c_void;
use inkwell::types::IntType;
use crate::module_system::ModuleSystem;
use crate::export_resolver::ExportResolver;


pub struct OopsieEntity<'ctx> {
    pub name: String,
    pub base: Option<String>,
    pub struct_type: inkwell::types::StructType<'ctx>,
    pub fields: Vec<(String, inkwell::types::BasicTypeEnum<'ctx>)>,
    pub methods: HashMap<String, FunctionValue<'ctx>>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub param_types: Vec<crate::ast::types::TypeDescriptor>,
}

#[derive(Clone)]
pub struct VarInfo<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub ty: BasicTypeEnum<'ctx>,
    pub is_const: bool,
    pub is_thread_state: bool, // 👈 new field
    pub entity_type: Option<String>, // 👈 NEW FIELD for entity dispatch
    pub object_type_name: Option<String>, // 👈 NEW FIELD for object type dispatch
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
    pub functions: HashMap<FunctionSignature, FunctionValue<'ctx>>,
    pub reverse_func_index: HashMap<String, Vec<FunctionSignature>>,
    pub entities: HashMap<String, OopsieEntity<'ctx>>,
    pub type_aliases: HashMap<String, crate::ast::types::ObjectTypeDefinition>, // ✅ NEW: Type alias registry
    pub wms: Option<Arc<Mutex<ModuleSystem>>>,
    pub resolver: Option<Arc<Mutex<ExportResolver>>>,
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
        // Add wpp_str_concat declaration if missing
let ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
if self.module.get_function("wpp_str_concat").is_none() {
    let fn_ty = ptr_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
    self.module.add_function("wpp_str_concat", fn_ty, None);
}
// === Readline support ===
let i8_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
if self.module.get_function("wpp_readline").is_none() {
    let fn_ty = i8_ptr_ty.fn_type(&[], false);
    self.module.add_function("wpp_readline", fn_ty, None);
}


    }
}


impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, name: &str, base_dir: &str) -> Self {
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

    // ✅ Initialize Codegen struct
    let mut codegen = Self {
        context,
        module,
        builder,
        i32_type,
        vars: HashMap::new(),
        globals: HashMap::new(),
        loop_stack: Vec::new(),
        switch_stack: Vec::new(),
        exception_flag: Some(flag_global.as_pointer_value()),
        exception_value_i32: Some(exc_i32_global.as_pointer_value()),
        exception_value_str: Some(exc_str_global.as_pointer_value()),

        // 🧩 Multiple dispatch maps
        functions: HashMap::new(),
        reverse_func_index: HashMap::new(),
        entities: HashMap::new(),
        type_aliases: HashMap::new(), // ✅ NEW: Empty type alias registry
        wms: Some(Arc::new(Mutex::new(ModuleSystem::new(base_dir)))),
        resolver: Some(Arc::new(Mutex::new(ExportResolver::new()))),
    };

    // ✅ Register runtime externs immediately after initialization
    codegen.init_runtime_support();
    codegen.init_network_support();
    codegen.init_thread_support();
    codegen.init_mutex_support();

    println!("🧠 [init] Codegen ready with multiple dispatch support");

    // ✅ Return ready-to-use Codegen instance
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
pub fn init_thread_support(&self) {
    let void_ty = self.context.void_type();
    let i8_ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    // === Thread spawn (GC-managed) ===
    // void* wpp_thread_spawn_gc(void* fn_ptr)
    let spawn_gc_ty = i8_ptr.fn_type(&[i8_ptr.into()], false);
    self.module.add_function("wpp_thread_spawn_gc", spawn_gc_ty, None);

    // === Thread join ===
    // void wpp_thread_join(void* handle)
    let join_ty = void_ty.fn_type(&[i8_ptr.into()], false);
    self.module.add_function("wpp_thread_join", join_ty, None);

    // === Thread poll ===
    // i32 wpp_thread_poll(void* handle)
    let poll_ty = i32_ty.fn_type(&[i8_ptr.into()], false);
    self.module.add_function("wpp_thread_poll", poll_ty, None);

    // === ThreadState new ===
    // void* wpp_thread_state_new(i32 initial)
    let state_new_ty = i8_ptr.fn_type(&[i32_ty.into()], false);
    self.module.add_function("wpp_thread_state_new", state_new_ty, None);

    // === ThreadState get ===
    // void* wpp_thread_state_get(void* ptr)
    let state_get_ty = i8_ptr.fn_type(&[i8_ptr.into()], false);
    self.module.add_function("wpp_thread_state_get", state_get_ty, None);

    // === ThreadState set ===
    // void wpp_thread_state_set(void* ptr, i32 val)
    let state_set_ty = void_ty.fn_type(&[i8_ptr.into(), i32_ty.into()], false);
    self.module.add_function("wpp_thread_state_set", state_set_ty, None);

    // === Thread join all ===
    // void wpp_thread_join_all(void)
    let join_all_ty = void_ty.fn_type(&[], false);
    self.module.add_function("wpp_thread_join_all", join_all_ty, None);
}


pub fn init_mutex_support(&self) {
    let void_ty = self.context.void_type();
    let i8ptr  = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    // === GC-aware mutex new ===
    // void* wpp_mutex_new(i32 initial)
    let new_ty = i8ptr.fn_type(&[i32_ty.into()], false);
    self.module.add_function("wpp_mutex_new", new_ty, None);

    // === GC-aware mutex lock ===
    // void wpp_mutex_lock(void* handle, i32 thread_id)
    let lock_ty = void_ty.fn_type(&[i8ptr.into(), i32_ty.into()], false);
    self.module.add_function("wpp_mutex_lock", lock_ty, None);

    // === GC-aware mutex unlock ===
    // void wpp_mutex_unlock(void* handle)
    let unlock_ty = void_ty.fn_type(&[i8ptr.into()], false);
    self.module.add_function("wpp_mutex_unlock", unlock_ty, None);
}

    /// Declare known Rust FFI functions for imported Rust modules
    pub fn declare_rust_ffi_functions(&self) {
        let i8_ptr = self.context.i8_type().ptr_type(AddressSpace::default());
        let i32_ty = self.context.i32_type();
        let i64_ty = self.context.i64_type();
        let void_ty = self.context.void_type();

        // Helper macro to declare if not already declared
        macro_rules! declare_ffi {
            ($name:expr, $fn_type:expr) => {
                if self.module.get_function($name).is_none() {
                    self.module.add_function($name, $fn_type, None);
                    println!("  ✅ Declared FFI function '{}'", $name);
                }
            };
        }

        // JSON library functions
        declare_ffi!("json_parse", i8_ptr.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("json_stringify", i8_ptr.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("json_pretty", i8_ptr.fn_type(&[i8_ptr.into(), i32_ty.into()], false));
        declare_ffi!("json_validate", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("json_get", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("json_get_string", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("json_get_int", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("json_merge", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("json_free", void_ty.fn_type(&[i8_ptr.into()], false));

        // File I/O library functions
        declare_ffi!("io_read_file", i8_ptr.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_write_file", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("io_read_bytes", i8_ptr.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_read_lines", i8_ptr.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_append_file", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("io_write_bytes", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("io_exists", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_delete_file", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_copy_file", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("io_rename_file", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("io_file_size", i64_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_is_file", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_is_dir", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_create_dir", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_create_dir_all", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_remove_dir", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_remove_dir_all", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_list_dir", i8_ptr.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("io_free", void_ty.fn_type(&[i8_ptr.into()], false));

        // CORS library functions
        declare_ffi!("cors_strlen", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("cors_int_to_string", i8_ptr.fn_type(&[i32_ty.into()], false));
        declare_ffi!("cors_contains", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cors_strcmp", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cors_strcasecmp", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cors_is_origin_allowed", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cors_is_method_allowed", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cors_are_headers_allowed", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cors_is_preflight", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("cors_gc_collect", i32_ty.fn_type(&[], false));
        declare_ffi!("cors_gc_shutdown", void_ty.fn_type(&[], false));

        // MySQL database driver functions
        declare_ffi!("mysql_connect", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mysql_close", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("mysql_query", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mysql_execute", i64_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mysql_prepare", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mysql_bind_execute", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mysql_begin_transaction", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("mysql_commit", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("mysql_rollback", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("mysql_get_last_error", i8_ptr.fn_type(&[], false));
        declare_ffi!("mysql_gc_collect", i32_ty.fn_type(&[], false));
        declare_ffi!("mysql_gc_shutdown", void_ty.fn_type(&[], false));

        // PostgreSQL database driver functions
        declare_ffi!("pg_connect", i8_ptr.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("pg_close", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("pg_query", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("pg_execute", i64_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("pg_begin_transaction", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("pg_commit", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("pg_rollback", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("pg_get_last_error", i8_ptr.fn_type(&[], false));
        declare_ffi!("pg_gc_collect", i32_ty.fn_type(&[], false));
        declare_ffi!("pg_gc_shutdown", void_ty.fn_type(&[], false));

        // MongoDB database driver functions
        declare_ffi!("mongo_connect", i8_ptr.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("mongo_close", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("mongo_find", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mongo_insert_one", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mongo_update_one", i64_ty.fn_type(&[i8_ptr.into(), i8_ptr.into(), i8_ptr.into(), i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mongo_delete_one", i64_ty.fn_type(&[i8_ptr.into(), i8_ptr.into(), i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("mongo_get_last_error", i8_ptr.fn_type(&[], false));
        declare_ffi!("mongo_gc_collect", i32_ty.fn_type(&[], false));
        declare_ffi!("mongo_gc_shutdown", void_ty.fn_type(&[], false));

        // Firebase database driver functions
        declare_ffi!("firebase_init", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("firebase_close", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("firebase_set", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("firebase_get", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("firebase_delete", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("firebase_get_last_error", i8_ptr.fn_type(&[], false));
        declare_ffi!("firebase_gc_collect", i32_ty.fn_type(&[], false));
        declare_ffi!("firebase_gc_shutdown", void_ty.fn_type(&[], false));

        // Apache Cassandra database driver functions
        declare_ffi!("cassandra_connect", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cassandra_close", i32_ty.fn_type(&[i8_ptr.into()], false));
        declare_ffi!("cassandra_query", i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cassandra_execute", i32_ty.fn_type(&[i8_ptr.into(), i8_ptr.into()], false));
        declare_ffi!("cassandra_get_last_error", i8_ptr.fn_type(&[], false));
        declare_ffi!("cassandra_gc_collect", i32_ty.fn_type(&[], false));
        declare_ffi!("cassandra_gc_shutdown", void_ty.fn_type(&[], false));
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

     Expr::StringLiteral(s) => {
    use inkwell::values::BasicValueEnum;

    // ✅ Create null-terminated bytes
    let cstring = std::ffi::CString::new(s.as_str()).unwrap();
    let bytes = cstring.as_bytes_with_nul();
    let array_type = self.context.i8_type().array_type(bytes.len() as u32);

    // ✅ Unique global name
    static mut STRING_ID: usize = 0;
    let name = unsafe {
        let id = STRING_ID;
        STRING_ID += 1;
        format!("strlit_{}", id)
    };

    // ✅ Create global constant
    let global = self.module.add_global(array_type, None, &name);
    let init = self.context.i8_type().const_array(
        &bytes
            .iter()
            .map(|&b| self.context.i8_type().const_int(b as u64, false))
            .collect::<Vec<_>>(),
    );
    global.set_initializer(&init);
    global.set_constant(true);
    global.set_linkage(inkwell::module::Linkage::Private);

    // ✅ Get a pointer to the first byte (i8*)
    let zero = self.context.i32_type().const_zero();
    let ptr = unsafe {
    self.builder
        .build_in_bounds_gep(
            array_type,                  // ✅ pointee type
            global.as_pointer_value(),   // ✅ pointer to [N x i8]
            &[zero, zero],               // ✅ indices [0,0]
            "str_gep",
        )
        .expect("Failed to build GEP for string literal")
};


    // ✅ Return as a BasicValueEnum (i8*)
    ptr.as_basic_value_enum()
}





        // === Variable lookup ===
       Expr::Variable(name) => {
    // 🧭 Try local or global variable first
    if let Some(var) = self.vars.get(name).or_else(|| self.globals.get(name)) {
        let val = self
            .builder
            .build_load(var.ty, var.ptr, &format!("load_{}", name))
            .unwrap();

        // 🧵 Only call wpp_thread_state_get if this var is a thread-state handle
        if var.is_thread_state {
            if let Some(get_fn) = self.module.get_function("wpp_thread_state_get") {
                let call = self
                    .builder
                    .build_call(get_fn, &[val.into()], "call_thread_state_get")
                    .unwrap();
                return call.try_as_basic_value().left().unwrap();
            }
        }

        // Normal load return
        return val.into();
    }

    // 🌐 Fallback: treat as function reference (for useThread, server.register, etc.)
    if let Some(sigs) = self.reverse_func_index.get(name) {
        if let Some(first_sig) = sigs.first() {
            if let Some(func) = self.functions.get(first_sig) {
                let fn_ptr: PointerValue<'_> = func.as_global_value().as_pointer_value();
                return fn_ptr.as_basic_value_enum();
            }
        }
    }

    panic!("Unknown variable or function: {}", name);
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
        let rhs_val = self.compile_expr(right.as_ref());

        // Lookup variable info (either local or global)
        if let Some(var) = self.vars.get(var_name).or_else(|| self.globals.get(var_name)) {
            println!("🔍 Found variable {} (is_const = {})", var_name, var.is_const);
            if var.is_const {
                panic!("❌ Cannot assign to constant variable '{}'", var_name);
            }

            let var_ty = var.ty;

            // ✅ Special case: Thread state pointer
            if let BasicTypeEnum::PointerType(_) = var_ty {
                println!("🧵 Detected thread state pointer assignment for {}", var_name);

                if let Some(set_fn) = self.module.get_function("wpp_thread_state_set") {
                    // Load pointer value (this is the actual thread-state handle)
                    let ptr_val = self
                        .builder
                        .build_load(var_ty, var.ptr, "load_thread_ptr")
                        .unwrap();

                    // Ensure RHS is integer (thread state stores i32s)
                    let rhs_int = match rhs_val {
                        BasicValueEnum::IntValue(iv) => iv,
                        _ => panic!(
                            "❌ Thread state set expected i32, got {:?}",
                            rhs_val.get_type()
                        ),
                    };

                    // Emit: call void @wpp_thread_state_set(ptr, i32)
                    self.builder
                        .build_call(
                            set_fn,
                            &[ptr_val.into(), rhs_int.into()],
                            "call_thread_state_set",
                        )
                        .unwrap();

                    return self.i32_type.const_int(0, false).into();
                } else {
                    panic!("❌ Missing runtime function: wpp_thread_state_set");
                }
            }

            // ✅ Normal value assignment (non-thread variables)
            let casted_val: BasicValueEnum<'ctx> = match (rhs_val, var_ty) {
                // Integer → Integer
                (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(int_ty)) => {
                    let rhs_bits = iv.get_type().get_bit_width();
                    let lhs_bits = int_ty.get_bit_width();

                    if rhs_bits == lhs_bits {
                        iv.as_basic_value_enum()
                    } else if rhs_bits < lhs_bits {
                        self.builder
                            .build_int_z_extend(iv, int_ty, "assign_zext")
                            .unwrap()
                            .as_basic_value_enum()
                    } else {
                        self.builder
                            .build_int_truncate(iv, int_ty, "assign_trunc")
                            .unwrap()
                            .as_basic_value_enum()
                    }
                }

                // Float → Float
                (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(_)) => fv.into(),

                // Int → Float
                (BasicValueEnum::IntValue(iv), BasicTypeEnum::FloatType(f_ty)) => {
                    self.builder
                        .build_signed_int_to_float(iv, f_ty, "assign_int2float")
                        .unwrap()
                        .into()
                }

                // Float → Int
                (BasicValueEnum::FloatValue(fv), BasicTypeEnum::IntType(i_ty)) => {
                    self.builder
                        .build_float_to_signed_int(fv, i_ty, "assign_float2int")
                        .unwrap()
                        .into()
                }

                // Pointer → Pointer
                (BasicValueEnum::PointerValue(pv), BasicTypeEnum::PointerType(_)) => pv.into(),

                (val, _) => panic!(
                    "❌ Type mismatch assigning {:?} to {:?}",
                    val.get_type(),
                    var_ty
                ),
            };

            self.builder.build_store(var.ptr, casted_val).unwrap();
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






let result: BasicValueEnum<'ctx> = match (&left_raw, &right_raw) {
    (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) if op == "+" => {
    // 🧩 Runtime string concatenation
    let ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
    let concat_fn = self.module.get_function("wpp_str_concat").unwrap_or_else(|| {
        let fn_ty = ptr_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        self.module.add_function("wpp_str_concat", fn_ty, None)
    });

    // 🧠 Build the call (pass raw pointer values)
    let call = self.builder
        .build_call(concat_fn, &[lp.clone().into(), rp.clone().into()], "concat")
        .expect("Failed to call wpp_str_concat");

    // ✅ Extract pointer result (the concatenated string)
    let result = call.try_as_basic_value()
        .left()
        .expect("wpp_str_concat should return a pointer");

    result
}
    // --- Integer + Integer ---
    // --- Integer + Integer ---
(BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
    match op.as_str() {
        // ✅ Boolean logic first
       "and" | "or" => {
    // 🧠 Ensure both operands are i1 for logic ops
    let lhs_val = *l;
    let rhs_val = *r;

    // If left operand is i32, convert to bool (i1)
    let lhs_bool = if lhs_val.get_type().get_bit_width() != 1 {
        self.builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                lhs_val,
                self.i32_type.const_int(0, false),
                "lhs_bool_cast",
            )
            .unwrap()
    } else {
        lhs_val
    };

    // If right operand is i32, convert to bool (i1)
    let rhs_bool = if rhs_val.get_type().get_bit_width() != 1 {
        self.builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                rhs_val,
                self.i32_type.const_int(0, false),
                "rhs_bool_cast",
            )
            .unwrap()
    } else {
        rhs_val
    };

    // ✅ Build AND/OR using i1s
    let result = if op == "and" {
        self.builder.build_and(lhs_bool, rhs_bool, "andtmp").unwrap()
    } else {
        self.builder.build_or(lhs_bool, rhs_bool, "ortmp").unwrap()
    };

    // ✅ Extend i1 → i32 for consistent numeric behavior
    // ✅ Keep as i1 (no extension) for proper bool return type
return result.as_basic_value_enum();

}


        // ✅ Arithmetic and comparisons
        "+" => self.builder.build_int_add(*l, *r, "addtmp").unwrap().as_basic_value_enum(),
        "-" => self.builder.build_int_sub(*l, *r, "subtmp").unwrap().as_basic_value_enum(),
        "*" => self.builder.build_int_mul(*l, *r, "multmp").unwrap().as_basic_value_enum(),
        "/" => self.builder.build_int_signed_div(*l, *r, "divtmp").unwrap().as_basic_value_enum(),

        "==" => self.builder.build_int_compare(inkwell::IntPredicate::EQ, *l, *r, "eqtmp").unwrap().as_basic_value_enum(),
        "!=" => self.builder.build_int_compare(inkwell::IntPredicate::NE, *l, *r, "netmp").unwrap().as_basic_value_enum(),
        "<"  => self.builder.build_int_compare(inkwell::IntPredicate::SLT, *l, *r, "lttmp").unwrap().as_basic_value_enum(),
        "<=" => self.builder.build_int_compare(inkwell::IntPredicate::SLE, *l, *r, "letmp").unwrap().as_basic_value_enum(),
        ">"  => self.builder.build_int_compare(inkwell::IntPredicate::SGT, *l, *r, "gttmp").unwrap().as_basic_value_enum(),
        ">=" => self.builder.build_int_compare(inkwell::IntPredicate::SGE, *l, *r, "getmp").unwrap().as_basic_value_enum(),

        _ => panic!("Unsupported integer operator: {}", op),
    }
}


    // --- Float + Float ---
    (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => match op.as_str() {
        "+" => self.builder.build_float_add(lf.clone(), rf.clone(), "fadd").unwrap().as_basic_value_enum(),
        "-" => self.builder.build_float_sub(lf.clone(), rf.clone(), "fsub").unwrap().as_basic_value_enum(),
        "*" => self.builder.build_float_mul(lf.clone(), rf.clone(), "fmul").unwrap().as_basic_value_enum(),
        "/" => self.builder.build_float_div(lf.clone(), rf.clone(), "fdiv").unwrap().as_basic_value_enum(),
        "==" => self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, lf.clone(), rf.clone(), "feq").unwrap().as_basic_value_enum(),
        "!=" => self.builder.build_float_compare(inkwell::FloatPredicate::ONE, lf.clone(), rf.clone(), "fne").unwrap().as_basic_value_enum(),
        "<"  => self.builder.build_float_compare(inkwell::FloatPredicate::OLT, lf.clone(), rf.clone(), "flt").unwrap().as_basic_value_enum(),
        "<=" => self.builder.build_float_compare(inkwell::FloatPredicate::OLE, lf.clone(), rf.clone(), "fle").unwrap().as_basic_value_enum(),
        ">"  => self.builder.build_float_compare(inkwell::FloatPredicate::OGT, lf.clone(), rf.clone(), "fgt").unwrap().as_basic_value_enum(),
        ">=" => self.builder.build_float_compare(inkwell::FloatPredicate::OGE, lf.clone(), rf.clone(), "fge").unwrap().as_basic_value_enum(),
        _ => panic!("Unsupported float operator: {}", op),
    },

    // --- Mixed: Int + Float ---
    (BasicValueEnum::IntValue(iv), BasicValueEnum::FloatValue(fv)) => {
        let f_ty = fv.get_type();
        let iv_cast = self.builder.build_signed_int_to_float(iv.clone(), f_ty, "int_to_float_lhs").unwrap();
        let new_expr = self.builder.build_float_add(iv_cast, fv.clone(), "promoted_add").unwrap();
        if ["+", "-", "*", "/"].contains(&op.as_str()) {
            new_expr.as_basic_value_enum()
        } else {
            panic!("Cannot compare mixed int/float types without explicit cast")
        }
    }

    (BasicValueEnum::FloatValue(fv), BasicValueEnum::IntValue(iv)) => {
        let f_ty = fv.get_type();
        let iv_cast = self.builder.build_signed_int_to_float(iv.clone(), f_ty, "int_to_float_rhs").unwrap();
        let new_expr = self.builder.build_float_add(fv.clone(), iv_cast, "promoted_add").unwrap();
        if ["+", "-", "*", "/"].contains(&op.as_str()) {
            new_expr.as_basic_value_enum()
        } else {
            panic!("Cannot compare mixed float/int types without explicit cast")
        }
    }
   // --- String (ptr) + String (ptr) ---
    (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) => {
        match op.as_str() {
            "==" | "!=" => {
                // Use strcmp for string comparison
                let i32_ty = self.context.i32_type();
                let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());

                let strcmp_fn = self.module.get_function("strcmp").unwrap_or_else(|| {
                    let ty = i32_ty.fn_type(&[i8ptr.into(), i8ptr.into()], false);
                    self.module.add_function("strcmp", ty, None)
                });

                let cmp_result = self.builder
                    .build_call(strcmp_fn, &[(*lp).into(), (*rp).into()], "strcmp")
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .unwrap()
                    .into_int_value();

                let zero = i32_ty.const_int(0, false);
                let is_equal = self.builder
                    .build_int_compare(inkwell::IntPredicate::EQ, cmp_result, zero, "streq")
                    .unwrap();

                if op == "==" {
                    is_equal.as_basic_value_enum()
                } else {
                    // != means not equal
                    let is_not_equal = self.builder
                        .build_not(is_equal, "strne")
                        .unwrap();
                    is_not_equal.as_basic_value_enum()
                }
            }
            "+" => {
                // String concatenation using wpp_str_concat
                let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());

                let concat_fn = self.module.get_function("wpp_str_concat").unwrap_or_else(|| {
                    let ty = i8ptr.fn_type(&[i8ptr.into(), i8ptr.into()], false);
                    self.module.add_function("wpp_str_concat", ty, None)
                });

                self.builder
                    .build_call(concat_fn, &[(*lp).into(), (*rp).into()], "strconcat")
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
            _ => panic!("❌ Unsupported string operator: {}", op),
        }
    }

// --- Boolean logic (and / or) ---




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

    // Unified basic-type printer: wpp_print_value_basic(ptr, i32)
    let wpp_print_value_basic = self.module.get_function("wpp_print_value_basic").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i8ptr.into(), i32_ty.into()], false);
        self.module.add_function("wpp_print_value_basic", ty, None)
    });

    // Specialized printers for arrays and objects
    let wpp_print_array = self.module.get_function("wpp_print_array").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i8ptr.into()], false);
        self.module.add_function("wpp_print_array", ty, None)
    });
    let wpp_print_object = self.module.get_function("wpp_print_object").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i8ptr.into()], false);
        self.module.add_function("wpp_print_object", ty, None)
    });

    // Declare printf for newline
    let printf_fn = self.module.get_function("printf").unwrap_or_else(|| {
        let ty = i32_ty.fn_type(&[i8ptr.into()], true);
        self.module.add_function("printf", ty, None)
    });

    // === Compile all arguments ===
    if args.is_empty() {
        panic!("print() expects at least one argument");
    }

    // Print each argument with a space separator
    for (i, arg) in args.iter().enumerate() {
        let val = self.compile_expr(arg);

        // === Handle based on value type ===
        match val {
            BasicValueEnum::IntValue(iv) => {
                // 🧮 Detect width to choose between i32/i64/bool
                let width = iv.get_type().get_bit_width();
                let type_id = match width {
                    1 => 5,    // bool
                    64 => 2,   // i64
                    _ => 1,    // i32 default
                };

                // Store to temporary alloca to pass by pointer
                let tmp = self.builder.build_alloca(iv.get_type(), "tmp_int").unwrap();
                self.builder.build_store(tmp, iv).unwrap();
                let casted = self.builder.build_pointer_cast(tmp, i8ptr, "casted_int").unwrap();

                self.builder
                    .build_call(
                        wpp_print_value_basic,
                        &[casted.into(), i32_ty.const_int(type_id, false).into()],
                        "call_print_basic_int",
                    )
                    .unwrap();
            }

            BasicValueEnum::FloatValue(fv) => {
                let f64_ty = self.context.f64_type();
                let type_id: u64 = if fv.get_type() == f64_ty {
                    4 // f64
                } else {
                    3 // f32
                };

                let tmp = self.builder.build_alloca(fv.get_type(), "tmp_float").unwrap();
                self.builder.build_store(tmp, fv).unwrap();
                let casted = self.builder.build_pointer_cast(tmp, i8ptr, "casted_float").unwrap();

                self.builder
                    .build_call(
                        wpp_print_value_basic,
                        &[casted.into(), i32_ty.const_int(type_id, false).into()],
                        "call_print_basic_float",
                    )
                    .unwrap();
            }

            BasicValueEnum::PointerValue(pv) => {
                // 🔤 Assume string (C char*) by default
                let type_id = i32_ty.const_int(6, false);
                self.builder
                    .build_call(
                        wpp_print_value_basic,
                        &[pv.into(), type_id.into()],
                        "call_print_basic_ptr",
                    )
                    .unwrap();
            }

            BasicValueEnum::ArrayValue(av) => {
                let tmp = self.builder.build_alloca(av.get_type(), "tmp_arr").unwrap();
                self.builder.build_store(tmp, av).unwrap();
                let casted = self.builder.build_pointer_cast(tmp, i8ptr, "casted_arr").unwrap();
                self.builder
                    .build_call(
                        wpp_print_array,
                        &[casted.into()],
                        "call_print_array",
                    )
                    .unwrap();
            }

            BasicValueEnum::StructValue(sv) => {
                let tmp = self.builder.build_alloca(sv.get_type(), "tmp_obj").unwrap();
                self.builder.build_store(tmp, sv).unwrap();
                let casted = self.builder.build_pointer_cast(tmp, i8ptr, "casted_obj").unwrap();
                self.builder
                    .build_call(
                        wpp_print_object,
                        &[casted.into()],
                        "call_print_object",
                    )
                    .unwrap();
            }

            _ => {
                println!("⚠️ Unsupported print type — using null");
                let null_ptr = i8ptr.const_null();
                self.builder
                    .build_call(
                        wpp_print_value_basic,
                        &[null_ptr.into(), i32_ty.const_int(0, false).into()],
                        "call_print_value_null",
                    )
                    .unwrap();
            }
        }
    }

    // Print newline at the end
    let newline_str = self.builder.build_global_string_ptr("\n", "newline").unwrap();
    self.builder
        .build_call(
            printf_fn,
            &[newline_str.as_pointer_value().into()],
            "print_newline",
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
// === HTTP STATUS ===
else if name == "http.status" {
    if args.len() != 1 {
        panic!("http.status(handle) expects 1 argument");
    }

    let i32_ty = self.context.i32_type();
    let handle = self.compile_expr(&args[0]);

    let fnc = self.module.get_function("wpp_http_status").unwrap_or_else(|| {
        let ty = i32_ty.fn_type(&[i32_ty.into()], false);
        self.module.add_function("wpp_http_status", ty, None)
    });

    let call = self.builder
        .build_call(fnc, &[handle.into()], "call_http_status")
        .unwrap();

    return call.try_as_basic_value().left().unwrap();
}

// === HTTP BODY ===
else if name == "http.body" {
    if args.len() != 1 {
        panic!("http.body(handle) expects 1 argument");
    }

    let i32_ty = self.context.i32_type();
    let i8ptr_ty = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
    let handle = self.compile_expr(&args[0]);

    let fnc = self.module.get_function("wpp_http_body").unwrap_or_else(|| {
        let ty = i8ptr_ty.fn_type(&[i32_ty.into()], false);
        self.module.add_function("wpp_http_body", ty, None)
    });

    let call = self.builder
        .build_call(fnc, &[handle.into()], "call_http_body")
        .unwrap();

    return call.try_as_basic_value().left().unwrap();
}

// === HTTP HEADERS ===
else if name == "http.headers" {
    if args.len() != 1 {
        panic!("http.headers(handle) expects 1 argument");
    }

    let i32_ty = self.context.i32_type();
    let i8ptr_ty = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
    let handle = self.compile_expr(&args[0]);

    let fnc = self.module.get_function("wpp_http_headers").unwrap_or_else(|| {
        let ty = i8ptr_ty.fn_type(&[i32_ty.into()], false);
        self.module.add_function("wpp_http_headers", ty, None)
    });

    let call = self.builder
        .build_call(fnc, &[handle.into()], "call_http_headers")
        .unwrap();

    return call.try_as_basic_value().left().unwrap();
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

    // 🧠 Resolve handler function via multiple dispatch table
let handler_fn: &FunctionValue<'_> = if let Some(sigs) = self.reverse_func_index.get(&handler_name) {
    if let Some(first_sig) = sigs.first() {
        self.functions.get(first_sig).unwrap_or_else(|| {
            panic!("Unknown handler function '{}'", handler_name)
        })
    } else {
        panic!("No overloads registered for handler '{}'", handler_name);
    }
} else {
    panic!("Unknown handler function '{}'", handler_name);
};

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


// === THREAD: useThread(fn) ===
else if name == "useThread" {
    if args.is_empty() {
        panic!("useThread(fn [, detached]) requires at least one argument");
    }

    let fn_ptr_val = self.compile_expr(&args[0]);
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let void_ty = self.context.void_type();
    let i32_ty = self.context.i32_type();

    // Optional detached flag
    let detached_flag = if args.len() > 1 {
        let val = self.compile_expr(&args[1]);
        match val {
            BasicValueEnum::IntValue(iv) => iv,
            _ => panic!("useThread(fn, detached) expects bool/int flag"),
        }
    } else {
        i32_ty.const_int(0, false)
    };

    // externs
    let spawn_fn = self.module.get_function("wpp_thread_spawn_gc").unwrap_or_else(|| {
        let ty = i8ptr.fn_type(&[i8ptr.into()], false);
        self.module.add_function("wpp_thread_spawn_gc", ty, None)
    });
    let join_fn = self.module.get_function("wpp_thread_join").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i8ptr.into()], false);
        self.module.add_function("wpp_thread_join", ty, None)
    });

    // cast function pointer
    let casted_ptr = if let BasicValueEnum::PointerValue(pv) = fn_ptr_val {
        self.builder
            .build_pointer_cast(pv, i8ptr, "thread_fn_cast")
            .unwrap()
    } else {
        panic!("useThread() expects a function reference");
    };

    // spawn thread
    let thread_handle = self.builder
        .build_call(spawn_fn, &[casted_ptr.into()], "call_thread_spawn_gc")
        .unwrap()
        .try_as_basic_value()
        .left()
        .expect("spawn must return handle");

    // === Conditional join ===
    let current_block = self.builder.get_insert_block().unwrap();
    let parent_fn = current_block.get_parent().unwrap();

    let join_block = self.context.append_basic_block(parent_fn, "join_thread");
    let cont_block = self.context.append_basic_block(parent_fn, "cont_thread");

    let zero = i32_ty.const_int(0, false);
    let cond = self.builder
        .build_int_compare(inkwell::IntPredicate::EQ, detached_flag, zero, "should_join")
        .unwrap();

    self.builder
        .build_conditional_branch(cond, join_block, cont_block)
        .unwrap();

    // === join_thread ===
    self.builder.position_at_end(join_block);
    self.builder
        .build_call(join_fn, &[thread_handle.into()], "call_thread_join")
        .unwrap();
    self.builder.build_unconditional_branch(cont_block).unwrap();

    // === cont_thread ===
    self.builder.position_at_end(cont_block);

    // 🚫 DO NOT insert a return here!
    // Leave this block open so the next AST nodes (sleep, print, etc.) can continue.

    return self.i32_type.const_int(0, false).into();
}




// === THREAD: useThreadState(initial) ===
else if name == "useThreadState" {
    if args.len() != 1 {
        panic!("useThreadState(initial) requires one argument");
    }

    let init_val = self.compile_expr(&args[0]);
    let i32_ty = self.context.i32_type();
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());

    let state_new_fn = self.module.get_function("wpp_thread_state_new").unwrap_or_else(|| {
        let ty = i8ptr.fn_type(&[i32_ty.into()], false);
        self.module.add_function("wpp_thread_state_new", ty, None)
    });

    let call = self.builder
        .build_call(state_new_fn, &[init_val.into()], "call_thread_state_new")
        .unwrap();

    return call.try_as_basic_value().left().unwrap();
}
else if name == "getThreadState" {
    if args.len() != 1 {
        panic!("getThreadState(ptr) requires one argument");
    }

    let ptr_val = self.compile_expr(&args[0]);
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());

    let fn_get = self.module.get_function("wpp_thread_state_get")
        .expect("wpp_thread_state_get should exist in module");

    let call = self.builder
        .build_call(fn_get, &[ptr_val.into()], "call_thread_state_get")
        .unwrap();

    return call.try_as_basic_value().left().unwrap();
}

// === MUTEX: useMutex(initial) ===
else if name == "useMutex" {
    if args.len() != 1 {
        panic!("useMutex(initial) requires one argument");
    }

    let init_val = self.compile_expr(&args[0]);
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    let fn_new = self.module.get_function("wpp_mutex_new").unwrap_or_else(|| {
        let ty = i8ptr.fn_type(&[i32_ty.into()], false);
        self.module.add_function("wpp_mutex_new", ty, None)
    });

    let call = self.builder
        .build_call(fn_new, &[init_val.into()], "call_mutex_new")
        .unwrap();

    return call.try_as_basic_value().left().unwrap();
}

// === MUTEX: lock(mtx, threadId) ===
else if name == "lock" {
    if args.len() != 2 {
        panic!("lock(mutex, threadId) requires 2 arguments");
    }

    let mtx_val = self.compile_expr(&args[0]);
    let tid_val = self.compile_expr(&args[1]);
    let void_ty = self.context.void_type();
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    let fn_lock = self.module.get_function("wpp_mutex_lock").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i8ptr.into(), i32_ty.into()], false);
        self.module.add_function("wpp_mutex_lock", ty, None)
    });

    self.builder
        .build_call(fn_lock, &[mtx_val.into(), tid_val.into()], "call_mutex_lock")
        .unwrap();

    return self.i32_type.const_int(0, false).into();
}

// === MUTEX: unlock(mtx) ===
else if name == "unlock" {
    if args.len() != 1 {
        panic!("unlock(mutex) requires one argument");
    }

    let mtx_val = self.compile_expr(&args[0]);
    let void_ty = self.context.void_type();
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());

    let fn_unlock = self.module.get_function("wpp_mutex_unlock").unwrap_or_else(|| {
        let ty = void_ty.fn_type(&[i8ptr.into()], false);
        self.module.add_function("wpp_mutex_unlock", ty, None)
    });

    self.builder
        .build_call(fn_unlock, &[mtx_val.into()], "call_mutex_unlock")
        .unwrap();

    return self.i32_type.const_int(0, false).into();
}

// === READLINE ===
else if name == "readline" {
    // Declare the extern if missing
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let readline_fn = self.module.get_function("wpp_readline").unwrap_or_else(|| {
        let fn_ty = i8ptr.fn_type(&[], false);
        self.module.add_function("wpp_readline", fn_ty, None)
    });

    // Call the runtime function
    let call = self.builder
        .build_call(readline_fn, &[], "call_readline")
        .unwrap();

    let result = call
        .try_as_basic_value()
        .left()
        .expect("wpp_readline must return a pointer");

    return result;
}

// === STRING LENGTH ===
else if name == "strlen" {
    if args.len() != 1 {
        panic!("strlen() expects exactly 1 argument (string)");
    }

    let str_val = self.compile_expr(&args[0]);
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    // Declare strlen from C standard library
    let strlen_fn = self.module.get_function("strlen").unwrap_or_else(|| {
        let fn_ty = i32_ty.fn_type(&[i8ptr.into()], false);
        self.module.add_function("strlen", fn_ty, None)
    });

    let call = self.builder
        .build_call(strlen_fn, &[str_val.into()], "call_strlen")
        .unwrap();

    return call
        .try_as_basic_value()
        .left()
        .expect("strlen must return an integer");
}

// === INTEGER TO STRING ===
else if name == "int_to_string" || name == "to_string" {
    if args.len() != 1 {
        panic!("{}() expects exactly 1 argument (integer)", name);
    }

    let int_val = self.compile_expr(&args[0]);
    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
    let i32_ty = self.context.i32_type();

    // Declare wpp_int_to_string runtime function
    let int_to_str_fn = self.module.get_function("wpp_int_to_string").unwrap_or_else(|| {
        let fn_ty = i8ptr.fn_type(&[i32_ty.into()], false);
        self.module.add_function("wpp_int_to_string", fn_ty, None)
    });

    let call = self.builder
        .build_call(int_to_str_fn, &[int_val.into()], "call_int_to_string")
        .unwrap();

    return call
        .try_as_basic_value()
        .left()
        .expect("wpp_int_to_string must return a pointer");
}

// === INDIRECT FUNCTION CALL (lambda stored in variable) ===
if let Some(var_info) = self.vars.get(name) {
    // === Load the function pointer ===
    let fn_ptr = self.builder
        .build_load(var_info.ty, var_info.ptr, &format!("load_fnptr_{}", name))
        .unwrap()
        .into_pointer_value();

    // === Compile argument expressions ===
    let compiled_args: Vec<BasicMetadataValueEnum<'ctx>> = args
        .iter()
        .map(|arg| self.compile_expr(arg).into())
        .collect();

    // === Infer argument types ===
    let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = compiled_args
        .iter()
        .map(|arg| {
            let ty = arg.as_any_value_enum().get_type();
            if ty.is_int_type() {
                let int_ty = ty.into_int_type();
                if int_ty.get_bit_width() == 1 {
                    self.context.bool_type().into()
                } else {
                    self.i32_type.into()
                }
            } else if ty.is_float_type() {
                self.context.f32_type().into()
            } else if ty.is_pointer_type() {
                self.context.i8_type().ptr_type(AddressSpace::default()).into()
            } else {
                self.i32_type.into()
            }
        })
        .collect();

    // === Infer return type dynamically ===
    let ret_ty = if name.contains("f32") || name.contains("float") {
        self.context.f32_type().as_basic_type_enum()
    } else if name.contains("bool") {
        self.context.bool_type().as_basic_type_enum()
    } else if name.contains("ptr") || name.contains("string") {
        self.context.i8_type().ptr_type(AddressSpace::default()).as_basic_type_enum()
    } else {
        self.i32_type.as_basic_type_enum()
    };

    let fn_type = ret_ty.fn_type(&param_types, false);

    // === Build the indirect call ===
    let call_site = self.builder
        .build_indirect_call(fn_type, fn_ptr, &compiled_args, &format!("call_indirect_{}", name))
        .unwrap();

    // === Return proper type ===
    return call_site
        .try_as_basic_value()
        .left()
        .unwrap_or_else(|| match ret_ty {
            BasicTypeEnum::IntType(i) => i.const_int(0, false).into(),
            BasicTypeEnum::FloatType(f) => f.const_float(0.0).into(),
            BasicTypeEnum::PointerType(p) => p.const_null().into(),
            _ => self.i32_type.const_zero().into(),
        });
}


           else if self.reverse_func_index.contains_key(name) {
    // 🧩 Collect argument TypeDescriptors for overload resolution
    use crate::ast::types::TypeDescriptor;

    let arg_types: Vec<TypeDescriptor> = args
        .iter()
        .map(|a| {
            match a {
                Expr::StringLiteral(_) => TypeDescriptor::Primitive("ptr".to_string()),
                Expr::Literal(num) => {
                    // Check if it looks like an HTTP status code (100-599)
                    if *num >= 100 && *num < 600 {
                        TypeDescriptor::HttpStatusLiteral(*num as u16)
                    } else {
                        TypeDescriptor::Primitive("i32".to_string())
                    }
                }
                Expr::TypedLiteral { value, ty } if ty == "f64" => TypeDescriptor::Primitive("f64".to_string()),
                Expr::TypedLiteral { value, ty } if ty.starts_with("i") => {
                    // Check if it's an HTTP status code
                    if let Ok(num) = value.parse::<i32>() {
                        if num >= 100 && num < 600 {
                            TypeDescriptor::HttpStatusLiteral(num as u16)
                        } else {
                            TypeDescriptor::Primitive("i32".to_string())
                        }
                    } else {
                        TypeDescriptor::Primitive("i32".to_string())
                    }
                }
                Expr::Variable(var_name) => {
                    // Check if it's a known variable (local or global)
                    if let Some(var_info) = self.vars.get(var_name).or_else(|| self.globals.get(var_name)) {
                        // 🎯 Check for entity or object type first
                        if let Some(entity_name) = &var_info.entity_type {
                            return TypeDescriptor::Entity(entity_name.clone());
                        }
                        if let Some(obj_type_name) = &var_info.object_type_name {
                            return TypeDescriptor::ObjectType(obj_type_name.clone());
                        }

                        // Fall back to LLVM type inference
                        let var_ty = &var_info.ty;
                        if var_ty.is_int_type() {
                            TypeDescriptor::Primitive("i32".to_string())
                        } else if var_ty.is_pointer_type() {
                            TypeDescriptor::Primitive("ptr".to_string())
                        } else if var_ty.is_float_type() {
                            TypeDescriptor::Primitive("f64".to_string())
                        } else {
                            TypeDescriptor::Primitive("i32".to_string())
                        }
                    } else {
                        TypeDescriptor::Primitive("i32".to_string())
                    }
                }
                Expr::BinaryOp { left, right, .. } => {
                    let l = self.infer_expr_type(left);
                    let r = self.infer_expr_type(right);
                    if l.is_float_type() || r.is_float_type() {
                        TypeDescriptor::Primitive("f64".to_string())
                    } else {
                        TypeDescriptor::Primitive("i32".to_string())
                    }
                }
                Expr::NewInstance { entity, .. } => {
                    // Instance creation returns an entity type
                    TypeDescriptor::Entity(entity.clone())
                }
                Expr::ObjectLiteral { type_name, .. } => {
                    // If object has a type name, use it for dispatch
                    if let Some(obj_type) = type_name {
                        TypeDescriptor::ObjectType(obj_type.clone())
                    } else {
                        TypeDescriptor::Primitive("ptr".to_string())
                    }
                }
                _ => TypeDescriptor::Primitive("i32".to_string()),
            }
        })
        .collect();

    println!("💡 Inferred arg types for {}: {:?}", name, arg_types);

// 🧠 Normalize for compatible overloads (f64 → f32, etc.)
let normalized_arg_types: Vec<TypeDescriptor> = arg_types
    .iter()
    .map(|t| match t {
        TypeDescriptor::Primitive(ty) if ty == "f64" => TypeDescriptor::Primitive("f32".to_string()),
        _ => t.clone(),
    })
    .collect();

println!("💡 Normalized arg types for {}: {:?}", name, normalized_arg_types);

// 🕵️ Find best match using specificity ranking
let mut sig_opt = None;
if let Some(sigs) = self.reverse_func_index.get(name) {
    println!("🔍 Available signatures for '{}': {:?}", name, sigs.iter().map(|s| &s.param_types).collect::<Vec<_>>());

    // Try exact match first (highest priority)
    sig_opt = sigs.iter().find(|sig| sig.param_types == arg_types).cloned();

    // If no exact match, use specificity-based selection
    if sig_opt.is_none() {
        // Find all compatible signatures and rank by specificity
        let mut candidates: Vec<(&FunctionSignature, u32)> = sigs
            .iter()
            .filter_map(|sig| {
                // Check if signature is compatible with arg_types
                if sig.param_types.len() != arg_types.len() {
                    return None;
                }

                let mut is_compatible = true;
                let mut total_specificity = 0u32;

                for (sig_type, arg_type) in sig.param_types.iter().zip(arg_types.iter()) {
                    let matches = match (sig_type, arg_type) {
                        // Exact match
                        (a, b) if a == b => true,
                        // Any wildcard matches anything
                        (TypeDescriptor::Any, _) => true,
                        // HTTP status range matches literal in range
                        (TypeDescriptor::HttpStatusRange(min, max), TypeDescriptor::HttpStatusLiteral(code)) => {
                            *code >= *min && *code <= *max
                        }
                        // Normalized match (try normalized_arg_types)
                        _ => {
                            if let Some(normalized) = normalized_arg_types.get(sig.param_types.iter().position(|t| t == sig_type)?) {
                                sig_type == normalized
                            } else {
                                false
                            }
                        }
                    };

                    if !matches {
                        is_compatible = false;
                        break;
                    }

                    total_specificity += sig_type.specificity();
                }

                if is_compatible {
                    Some((sig, total_specificity))
                } else {
                    None
                }
            })
            .collect();

        if !candidates.is_empty() {
            // Sort by specificity (descending - higher is more specific)
            candidates.sort_by(|a, b| b.1.cmp(&a.1));
            sig_opt = Some(candidates[0].0.clone());
            println!("🎯 Selected overload by specificity: {:?} (score: {})", sig_opt.as_ref().unwrap().param_types, candidates[0].1);
        }
    }
}

if let Some(sig) = sig_opt {
    // 🔹 Borrow function temporarily (avoids long immutable borrow)
    let func_val = {
        let f = self.functions.get(&sig).unwrap();
        *f // FunctionValue<'ctx> implements Copy
    };

    // ✅ Borrow ended — safe to reuse self below

    // Compile arguments
    let mut compiled_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
    for a in args {
        let v = self.compile_expr(a);
        compiled_args.push(v.into());
    }

    println!("💥 Resolved call {}({:?})", sig.name, sig.param_types);

    // Call the correct overload (mangled name)
    let llvm_name = if sig.param_types.is_empty() {
        sig.name.clone()
    } else {
        let mangled_types: Vec<String> = sig.param_types.iter()
            .map(|td| td.to_mangle_string())
            .collect();
        format!("{}__{}", sig.name, mangled_types.join("_"))
    };

    let target_fn = self.module.get_function(&llvm_name).unwrap_or(func_val);
    println!("🧬 Using LLVM function: {}", llvm_name);
    // 🧠 Ensure argument types match function signature
let fn_param_types: Vec<_> = target_fn.get_type().get_param_types();

for (i, (arg_val, param_ty)) in compiled_args.iter_mut().zip(fn_param_types.iter()).enumerate() {
    let arg_ty = arg_val.clone().as_any_value_enum().get_type();

    

let arg_ty_basic = arg_ty.try_into().ok();
let param_ty_basic = Some(*param_ty);

if arg_ty_basic != param_ty_basic {
    // handle type conversion


    // Handle float double → float
    if arg_ty.is_float_type() && param_ty.is_float_type() {
        let arg_float_ty = arg_ty.into_float_type();
        let param_float_ty = param_ty.into_float_type();

        // Check if it’s actually f64 → f32
       let f32_ty = self.context.f32_type();
let f64_ty = self.context.f64_type();

if arg_float_ty == f64_ty && param_float_ty == f32_ty {
let float_val = arg_val
    .clone()
    .into_float_value(); // BasicMetadataValueEnum already implements into_float_value

let casted = self
    .builder
    .build_float_cast(
        float_val,
        param_float_ty,
        &format!("cast_f64_to_f32_arg{}", i),
    )
    .unwrap();

*arg_val = casted.into();




        }
    }

    // Handle i1 → i32 (bool promotion)
    else if arg_ty.is_int_type()
        && param_ty.is_int_type()
        && arg_ty.into_int_type().get_bit_width() == 1
        && param_ty.into_int_type().get_bit_width() == 32
    {
        // Convert argument to an IntValue safely
let int_val = arg_val
    .clone()
    .into_int_value(); // works for BasicMetadataValueEnum that holds int

// Perform zero-extend from i1 → i32
let casted = self
    .builder
    .build_int_z_extend(
        int_val,
        param_ty.into_int_type(),
        &format!("bool_to_i32_arg{}", i),
    )
    .unwrap();

// Replace the original arg with the extended version
*arg_val = casted.into();



    }
}

}

    let call_site = self
        .builder
        .build_call(target_fn, &compiled_args, &format!("call_{}", sig.name))
        .unwrap();

    call_site
        .try_as_basic_value()
        .left()
        .unwrap_or_else(|| self.i32_type.const_int(0, false).into())
} else {
    panic!("❌ No matching overload for {}({:?})", name, arg_types);
}

}

// === ENTITY QUALIFIED CALL FALLBACK ===
else if name.contains('.') {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() == 2 {
        let (lhs, method_name) = (parts[0], parts[1]);
        println!("🧭 Trying to resolve entity-qualified call: {} -> {}", lhs, method_name);

        // === 1️⃣ Case A: Direct static entity call (e.g. Dog.bark)
        let full_name_sig = FunctionSignature {
            name: name.clone(),
            param_types: vec![],
        };
        if let Some(func) = self.functions.get(&full_name_sig) {
            println!("✅ Direct entity-qualified match for {}", name);
            let call_site = self
                .builder
                .build_call(*func, &[], &format!("call_{}", name))
                .unwrap();
            return call_site
                .try_as_basic_value()
                .left()
                .unwrap_or_else(|| self.i32_type.const_int(0, false).into());
        }

       // === 2️⃣ Case B: Instance method call (e.g. d.bark)
if let Some(var_info) = self.vars.get(lhs) {
    if let Some(entity_type) = &var_info.entity_type {
        let resolved = format!("{}.{}", entity_type, method_name);
        println!("🔍 Resolved instance call '{}.{}' -> '{}'", lhs, method_name, resolved);

        let sig = FunctionSignature {
            name: resolved.clone(),
            param_types: vec![],
        };

        if let Some(func) = self.functions.get(&sig) {
    // ✅ Load the pointer stored in variable `lhs`
    let instance_ptr = self
        .builder
        .build_load(
            var_info.ty,             // <-- the type of what we're loading
            var_info.ptr,            // <-- the pointer variable itself
            &format!("{}_load", lhs) // <-- name for IR
        )
        .unwrap();

    // ✅ Pass `me` as the first argument
    let args: Vec<BasicMetadataValueEnum> = vec![instance_ptr.into()];

    let call_site = self
        .builder
        .build_call(*func, &args, &format!("call_{}", resolved))
        .unwrap();

    return call_site
        .try_as_basic_value()
        .left()
        .unwrap_or_else(|| self.i32_type.const_int(0, false).into());
}
else {
            panic!(
                "Method '{}' not found for entity type '{}'",
                method_name, entity_type
            );
        }
    }
}


        // === 3️⃣ Case C: Fallback search by unqualified name
        for (sig, func) in &self.functions {
            if sig.name == method_name {
                println!("🔗 Fallback matched unqualified {}.{}", lhs, method_name);
                let call_site = self
                    .builder
                    .build_call(*func, &[], &format!("call_{}", method_name))
                    .unwrap();
                return call_site
                    .try_as_basic_value()
                    .left()
                    .unwrap_or_else(|| self.i32_type.const_int(0, false).into());
            }
        }
    }

    panic!("Unknown entity-qualified function: {}", name);
}



            else {
                // === FFI FALLBACK: Check if function exists in module (Rust FFI) ===
                if let Some(func) = self.module.get_function(name) {
                    println!("🦀 Found FFI function '{}' in module", name);

                    // Compile arguments
                    let mut compiled_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
                    for a in args {
                        let v = self.compile_expr(a);
                        compiled_args.push(v.into());
                    }

                    // Call the FFI function
                    let call_site = self
                        .builder
                        .build_call(func, &compiled_args, &format!("call_{}", name))
                        .unwrap();

                    return call_site
                        .try_as_basic_value()
                        .left()
                        .unwrap_or_else(|| self.i32_type.const_int(0, false).into());
                } else {
                    panic!("Unknown function: {}", name);
                }
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
        is_thread_state: false,
        entity_type: None,
        object_type_name: None,
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







Expr::Funcy { name, params, body, is_async, params_patterns: _ } => {
    // 1️⃣ Compile the function (async or not)
    let func_val = if *is_async {
        self.compile_async_funcy(name, params, body)
    } else {
        self.compile_funcy(name, params, body, None, None)
    };

    // 2️⃣ Return a pointer to it as a first-class value
    func_val.as_global_value().as_pointer_value().into()
}




Expr::Return(expr_opt) => {
    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
    let func_name = func.get_name().to_str().unwrap_or_default().to_string();
    let func_ret_ty = func.get_type().get_return_type();

    // === Evaluate the expression ===
    let raw_val = if let Some(expr) = expr_opt {
        self.compile_expr(expr)
    } else if let Some(ret_ty) = func_ret_ty {
        match ret_ty {
            BasicTypeEnum::IntType(i) => i.const_int(0, false).into(),
            BasicTypeEnum::FloatType(f) => f.const_float(0.0).into(),
            BasicTypeEnum::PointerType(p) => p.const_null().into(),
            _ => self.i32_type.const_int(0, false).into(),
        }
    } else {
        self.i32_type.const_int(0, false).into()
    };

    // === Normalize the return type ===
    let ret_val = if let Some(ret_ty) = func_ret_ty {
        match ret_ty {
            BasicTypeEnum::IntType(i) => {
                if raw_val.is_int_value() {
                    raw_val
                } else if raw_val.is_float_value() {
                    self.builder
                        .build_float_to_signed_int(raw_val.into_float_value(), i, "ftoi")
                        .unwrap()
                        .into()
                } else {
                    i.const_int(0, false).into()
                }
            }
            BasicTypeEnum::FloatType(f) => {
                if raw_val.is_float_value() {
                    raw_val
                } else if raw_val.is_int_value() {
                    self.builder
                        .build_signed_int_to_float(raw_val.into_int_value(), f, "itof")
                        .unwrap()
                        .into()
                } else {
                    f.const_float(0.0).into()
                }
            }
            BasicTypeEnum::PointerType(_) => {
                if raw_val.is_pointer_value() {
                    raw_val
                } else {
                    let i8ptr = self.context.i8_type().ptr_type(AddressSpace::default());
                    i8ptr.const_null().into()
                }
            }
            _ => raw_val,
        }
    } else {
        raw_val
    };

    // === Async return signal ===
    if func_name != "bootstrap_main" {
        let void_ty = self.context.void_type();
        let i32_ty = self.i32_type;
        let i8ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());

        // Declare or get wpp_return(ptr, i32)
        let wpp_return = self.module.get_function("wpp_return").unwrap_or_else(|| {
            let fn_ty = void_ty.fn_type(&[i8ptr_ty.into(), i32_ty.into()], false);
            self.module.add_function("wpp_return", fn_ty, None)
        });

        // 🧠 Allocate a temporary slot to hold the return value
        let tmp_ptr = self.builder.build_alloca(ret_val.get_type(), "ret_tmp").unwrap();
        self.builder.build_store(tmp_ptr, ret_val).unwrap();

        // 🧩 Cast pointer to void*
        let void_ptr = self.builder
            .build_pointer_cast(tmp_ptr, i8ptr_ty, "ret_as_void_ptr")
            .unwrap();

        // 🏷️ Determine runtime type tag
        let type_tag = match ret_val.get_type() {
            BasicTypeEnum::IntType(i) if i.get_bit_width() == 1 => i32_ty.const_int(3, false), // bool
            BasicTypeEnum::IntType(_) => i32_ty.const_int(1, false), // i32
            BasicTypeEnum::FloatType(_) => i32_ty.const_int(2, false), // f32/f64
            BasicTypeEnum::PointerType(_) => i32_ty.const_int(4, false), // string/object ptr
            _ => i32_ty.const_int(0, false), // unknown
        };

        // 🚀 Call wpp_return(void*, i32)
        self.builder
            .build_call(wpp_return, &[void_ptr.into(), type_tag.into()], "async_return_signal")
            .unwrap();
    }

    // === Actual return ===
    self.builder.build_return(Some(&ret_val)).unwrap();

    // === Move builder to a safe continuation block ===
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

Expr::ObjectLiteral { fields, type_name: _ } => {
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


Expr::NewInstance { entity, args } => {
    println!("🐾 Allocating new instance of entity: {}", entity);

    // 1️⃣ Retrieve entity definition
    let oopsie = self.entities.get(entity)
        .unwrap_or_else(|| panic!("Unknown entity '{}'", entity));

    let struct_ty = oopsie.struct_type;

    // 2️⃣ Allocate memory (via malloc or GC)
    let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
        self.module.add_function(
            "malloc",
            self.context.i8_type().ptr_type(AddressSpace::default())
                .fn_type(&[self.i32_type.into()], false),
            None,
        )
    });

    let size_val = struct_ty.size_of().unwrap();
    let size_i32 = self.builder
        .build_int_cast(size_val, self.i32_type, "size_i32")
        .unwrap();

    let raw_ptr = self.builder
        .build_call(malloc_fn, &[size_i32.into()], "alloc_instance")
        .unwrap()
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_pointer_value();

    // 3️⃣ Cast pointer to entity type
    let typed_ptr = self.builder
        .build_bitcast(raw_ptr, struct_ty.ptr_type(AddressSpace::default()), "as_struct")
        .unwrap()
        .into_pointer_value();

    // 4️⃣ Initialize fields (defaults or zero)
    for (i, (field_name, field_ty)) in oopsie.fields.iter().enumerate() {
        let field_ptr = unsafe {
            self.builder
                .build_struct_gep(struct_ty, typed_ptr, i as u32, field_name)
                .unwrap()
        };

        // TODO: default initialization from entity declaration (for now, zero)
        let zero_val: BasicValueEnum<'ctx> = match field_ty {
    BasicTypeEnum::IntType(i) => i.const_int(0, false).into(),
    BasicTypeEnum::FloatType(f) => f.const_float(0.0).into(),
    BasicTypeEnum::PointerType(p) => p.const_null().into(),
    _ => self.i32_type.const_zero().into(),
};

        self.builder.build_store(field_ptr, zero_val);
    }

    // 5️⃣ Optional: call constructor if it exists (Dog.new)
    let ctor_name = format!("{}.new", entity);
    let ctor_sig = FunctionSignature {
        name: ctor_name.clone(),
        param_types: args.iter().map(|_| crate::ast::types::TypeDescriptor::Primitive("i32".to_string())).collect(),
    };

    if let Some(func_val) = self.functions.get(&ctor_sig).cloned() {
    println!("🧩 Calling constructor '{}'", ctor_name);

    let mut compiled_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
    compiled_args.push(typed_ptr.into()); // 'me' argument

    // ✅ Borrow self mutably again AFTER immutable borrow ended
    for a in args {
        let compiled = self.compile_expr(a);
        compiled_args.push(compiled.into());
    }

    self.builder
        .build_call(func_val, &compiled_args, &format!("call_ctor_{}", entity))
        .unwrap();
} else {
    println!("⚙️ No constructor found for '{}', skipping init", entity);
}


    // ✅ Return pointer to the instance
    typed_ptr.into()
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
            Node::Entity(entity) => {
        self.compile_entity(entity);
        None // 👈 explicitly return None so the return type matches
    }
    Node::TypeAlias(type_def) => {
        // Register type alias for dispatch resolution
        println!("📝 Registering type alias: {}", type_def.name);
        self.type_aliases.insert(type_def.name.clone(), type_def.clone());
        None
    }
    Node::ImportAll { module } | Node::ImportList { module, .. } => {
        if module.starts_with("rust:") {
            println!("🦀 Declaring FFI functions for Rust module '{}'", module);
            self.declare_rust_ffi_functions();
        } else {
            println!("📦 Skipping import '{}': already resolved by WMS", module);
        }
        None
    }

    Node::Export { name, .. } => {
        println!("📤 Export '{}' handled by ExportResolver", name);
        None
    }

        Node::Let { name, value, is_const, ty } => {
    println!("🧱 Compiling top-level node: Let {{ name: {}, ty: {:?} }}", name, ty);

    // === Detect heap-allocated expressions (arrays/objects) ===
    let is_heap_value = matches!(value, Expr::ArrayLiteral(_) | Expr::ObjectLiteral { .. });
    if is_heap_value {
        println!("💾 Variable `{}` is a heap object — allocating as pointer", name);
    }

    // === Determine variable type ===
    // === Determine variable type ===
// === Determine variable type ===
let var_type: BasicTypeEnum<'ctx> = if is_heap_value {
    // 💾 All heap structures are stored as pointers (i8*)
    self.context
        .i8_type()
        .ptr_type(inkwell::AddressSpace::default())
        .as_basic_type_enum()

// 🧩 Case: Lambda (Funcy expression)
} else if matches!(value, Expr::Funcy { .. }) {
    // 🧠 Lambdas are compiled functions; store as a function pointer (i8*)
    self.context
        .i8_type()
        .ptr_type(inkwell::AddressSpace::default())
        .as_basic_type_enum()

// 🧵 Special case: if RHS is a function call that returns a pointer
} else if let Expr::Call { name, .. } = value {
    if name == "useThreadState"
        || name == "useMutex"
        || name == "useThread"
        || name == "http.body"
        || name == "http.headers"
        || name == "readline"
        || name == "int_to_string"
        || name == "to_string"
        // FFI functions that return strings (pointers)
        || name == "json_parse"
        || name == "json_stringify"
        || name == "json_pretty"
        || name == "json_get"
        || name == "json_get_string"
        || name == "json_merge"
        || name == "io_read_file"
        || name == "io_read_bytes"
        || name == "io_read_lines"
        || name == "io_list_dir"
        // CORS library pointer-returning functions
        || name == "cors_int_to_string"
        // MySQL database driver pointer-returning functions
        || name == "mysql_connect"
        || name == "mysql_query"
        || name == "mysql_prepare"
        || name == "mysql_bind_execute"
        || name == "mysql_get_last_error"
        // PostgreSQL database driver pointer-returning functions
        || name == "pg_connect"
        || name == "pg_query"
        || name == "pg_get_last_error"
        // MongoDB database driver pointer-returning functions
        || name == "mongo_connect"
        || name == "mongo_find"
        || name == "mongo_insert_one"
        || name == "mongo_get_last_error"
        // Firebase database driver pointer-returning functions
        || name == "firebase_init"
        || name == "firebase_get"
        || name == "firebase_get_last_error"
        // Cassandra database driver pointer-returning functions
        || name == "cassandra_connect"
        || name == "cassandra_query"
        || name == "cassandra_get_last_error"
    {
        // These builtins/FFI functions return pointers
        self.context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default())
            .as_basic_type_enum()
    } else {
        // Default scalar
        self.context.i32_type().as_basic_type_enum()
    }
}


 else if let Some(t) = ty {
    self.resolve_basic_type(t)

// 🧠 Type inference from RHS (literal-based)
} else {
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

            // 🔤 String literals should be stored as pointers
            Expr::StringLiteral(_) => {
                self.context
                    .i8_type()
                    .ptr_type(inkwell::AddressSpace::default())
                    .as_basic_type_enum()
            }

            _ => {
                println!("⚠️ Complex expression — defaulting to i32");
                self.i32_type.into()
            }
        }
    };
    // === Special case: entity instantiation (let d = new Dog(...)) ===
if let Expr::NewInstance { entity, args } = value {
    println!("🐾 Allocating new instance of entity: {}", entity);

    // Call helper to allocate + call constructor
    let instance_ptr = self.compile_new_instance(entity, args);

    // All entity instances are stored as i8* (generic object pointer)
    let entity_ptr_ty = self
        .context
        .i8_type()
        .ptr_type(inkwell::AddressSpace::default())
        .as_basic_type_enum();

    let alloca = self.builder.build_alloca(entity_ptr_ty, name).unwrap();
    self.builder.build_store(alloca, instance_ptr).unwrap();

    // ✅ Register variable in self.vars with entity type
    self.vars.insert(
        name.clone(),
        VarInfo {
            ptr: alloca,
            ty: entity_ptr_ty,
            is_const: *is_const,
            is_thread_state: false,
            entity_type: Some(entity.clone()), // 👈 NEW FIELD (add this to VarInfo)
            object_type_name: None,
        },
    );

    // ✅ skip the rest of normal let logic
    return None;
}

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

    // === Extract object type name if it's a typed object literal ===
    let obj_type = if let Expr::ObjectLiteral { type_name, .. } = value {
        type_name.clone()
    } else {
        None
    };

    // === Register variable ===
    self.vars.insert(
        name.clone(),
        VarInfo {
            ptr: alloca,
            ty: var_type,
            is_const: *is_const,
            is_thread_state: false,
            entity_type: None,
            object_type_name: obj_type,
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

    /// Helper: Extract TypeDescriptors from function parameters
    /// This checks params_patterns first (enhanced dispatch), falls back to basic type inference
    fn extract_param_type_descriptors(
        &self,
        params: &[String],
        params_patterns: &Option<Vec<crate::ast::types::ParameterPattern>>,
    ) -> Vec<crate::ast::types::TypeDescriptor> {
        use crate::ast::types::TypeDescriptor;

        // If we have enhanced patterns, use them
        if let Some(patterns) = params_patterns {
            return patterns
                .iter()
                .map(|param_pattern| {
                    if let Some(pattern) = &param_pattern.pattern {
                        // Extract TypeDescriptor from the pattern
                        if let Some(td) = pattern.to_type_descriptor() {
                            // Resolve if it's an entity or object type
                            match &td {
                                TypeDescriptor::ObjectType(name) => {
                                    // Check if it's actually an entity
                                    println!("🔍 Resolving ObjectType '{}' - entities: {:?}, type_aliases: {:?}",
                                        name,
                                        self.entities.keys().collect::<Vec<_>>(),
                                        self.type_aliases.keys().collect::<Vec<_>>()
                                    );
                                    if self.entities.contains_key(name) {
                                        println!("✅ Resolved '{}' as Entity", name);
                                        TypeDescriptor::Entity(name.clone())
                                    } else if self.type_aliases.contains_key(name) {
                                        println!("✅ Resolved '{}' as ObjectType", name);
                                        TypeDescriptor::ObjectType(name.clone())
                                    } else {
                                        println!("⚠️  '{}' not found - treating as Primitive", name);
                                        // Unknown type, treat as primitive
                                        TypeDescriptor::Primitive(name.clone())
                                    }
                                }
                                _ => td,
                            }
                        } else {
                            // Default to i32 if pattern doesn't have a type descriptor
                            TypeDescriptor::Primitive("i32".to_string())
                        }
                    } else {
                        // No pattern, infer from param name annotation
                        let param_str = &param_pattern.name;
                        if param_str.contains(':') {
                            let parts: Vec<&str> = param_str.split(':').collect();
                            if parts.len() == 2 {
                                let type_str = parts[1].trim();
                                TypeDescriptor::Primitive(type_str.to_string())
                            } else {
                                TypeDescriptor::Primitive("i32".to_string())
                            }
                        } else {
                            TypeDescriptor::Primitive("i32".to_string())
                        }
                    }
                })
                .collect();
        }

        // Fallback: parse from params strings (old format: "name:type")
        params
            .iter()
            .map(|p| {
                if p.contains(':') {
                    let parts: Vec<&str> = p.split(':').collect();
                    if parts.len() == 2 {
                        let type_str = parts[1].trim();
                        // Check if it's an entity or object type
                        if self.entities.contains_key(type_str) {
                            TypeDescriptor::Entity(type_str.to_string())
                        } else if self.type_aliases.contains_key(type_str) {
                            TypeDescriptor::ObjectType(type_str.to_string())
                        } else {
                            TypeDescriptor::Primitive(type_str.to_string())
                        }
                    } else {
                        TypeDescriptor::Primitive("i32".to_string())
                    }
                } else {
                    TypeDescriptor::Primitive("i32".to_string())
                }
            })
            .collect()
    }

/// Create main(), compile nodes, and return i32.
pub fn compile_main(&mut self, nodes: &[Node]) -> FunctionValue<'ctx> {
    // === Pre-pass: Compile entities and type aliases first (BEFORE module check) ===
    println!("🔍 Pre-pass: Processing {} nodes for entities and type aliases", nodes.len());
    for node in nodes {
        match node {
            Node::Entity(entity) => {
                println!("🏗️ Pre-pass found entity: {}", entity.name);
                self.compile_entity(entity);
            }
            Node::TypeAlias(type_def) => {
                self.type_aliases.insert(type_def.name.clone(), type_def.clone());
                println!("📝 Registered type alias: {}", type_def.name);
            }
            Node::ImportAll { module } | Node::ImportList { module, .. } => {
                if module.starts_with("rust:") {
                    println!("🦀 Pre-pass: Declaring FFI functions for Rust module '{}'", module);
                    self.declare_rust_ffi_functions();
                }
            }
            _ => {}
        }
    }

    // 🔒 Skip entrypoint scaffolding for submodules
    let module_name = self.module.get_name().to_str().unwrap_or_default();
    if module_name != "main" {
        println!("⚙️ Compiling submodule '{}' — skipping main_async/main wrappers", module_name);

        // Just compile function bodies; no entrypoint creation
        for node in nodes {
            if let Node::Expr(Expr::Funcy { name, params, body, is_async, params_patterns, .. }) = node {
                if *is_async {
                    self.compile_async_funcy(name, params, body);
                } else {
                    // Extract type descriptors for proper dispatch
                    let type_descriptors = self.extract_param_type_descriptors(params, params_patterns);
                    self.compile_funcy(name, params, body, Some(&type_descriptors), None);
                }
            }
        }

        // Return a dummy placeholder function to satisfy return type
        return self.module.add_function(
            "__submodule_stub",
            self.i32_type.fn_type(&[], false),
            None,
        );
    }

    // === Root module only ===
    let fn_type = self.i32_type.fn_type(&[], false);
    let async_fn = self.module.add_function("main_async", fn_type, None);
    let entry = self.context.append_basic_block(async_fn, "entry");
    self.builder.position_at_end(entry);

    // === Use a dedicated temporary builder for the entry ===
    let entry_builder = self.context.create_builder();
    entry_builder.position_at_end(entry);


    // === Exception slots (kept global) ===
    let flag_ptr = entry_builder.build_alloca(self.i32_type, "exc_flag").unwrap();
    entry_builder.build_store(flag_ptr, self.i32_type.const_int(0, false)).unwrap();

    let val_i32_ptr = entry_builder.build_alloca(self.i32_type, "exc_val_i32").unwrap();
    entry_builder.build_store(val_i32_ptr, self.i32_type.const_int(0, false)).unwrap();

    let str_ptr_ty = self.context.i8_type().ptr_type(AddressSpace::default());
    let val_str_ptr = entry_builder.build_alloca(str_ptr_ty, "exc_val_str").unwrap();
    entry_builder.build_store(val_str_ptr, str_ptr_ty.const_null()).unwrap();

    self.exception_flag = Some(flag_ptr);
    self.exception_value_i32 = Some(val_i32_ptr);
    self.exception_value_str = Some(val_str_ptr);
    if let (Some(wms_arc), Some(resolver_arc)) = (&self.wms, &self.resolver) {
    // 🔒 Lock both Arc<Mutex<T>> to get access to the inner values
    let wms = wms_arc.lock().unwrap();
    let mut resolver = resolver_arc.lock().unwrap();

    println!("🔗 [wms] Collecting exports from cached modules...");
    resolver.collect_exports(&wms);

    println!("🔗 [wms] Applying imports into current module...");
    resolver.apply_imports(&mut self.module, &wms);
}

  // === Predeclare functions ===
for node in nodes {
    if let Node::Expr(Expr::Funcy { name, params, body, params_patterns, .. }) = node {
        // Infer parameter types from body (minimal version)
        let mut int_params = std::collections::HashSet::new();
        let mut ptr_params = std::collections::HashSet::new();
        let mut contains_string_literal = false;

        fn scan_for_types(
            nodes: &[Node],
            int_params: &mut std::collections::HashSet<String>,
            ptr_params: &mut std::collections::HashSet<String>,
            contains_string_literal: &mut bool,
        ) {
            for node in nodes {
                match node {
                    Node::Expr(expr) => match expr {
                        Expr::BinaryOp { left, right, .. } => {
                            if let (Expr::StringLiteral(_), _) | (_, Expr::StringLiteral(_)) = (&**left, &**right) {
                                *contains_string_literal = true;
                            }
                            scan_for_types(&[Node::Expr(*left.clone())], int_params, ptr_params, contains_string_literal);
                            scan_for_types(&[Node::Expr(*right.clone())], int_params, ptr_params, contains_string_literal);
                        }
                        Expr::StringLiteral(_) => *contains_string_literal = true,
                        Expr::Call { args, .. } => {
                            for a in args {
                                scan_for_types(&[Node::Expr(a.clone())], int_params, ptr_params, contains_string_literal);
                            }
                        }
                        Expr::Return(inner) => {
                            if let Some(inner_expr) = inner {
                                scan_for_types(&[Node::Expr(*inner_expr.clone())], int_params, ptr_params, contains_string_literal);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        scan_for_types(body, &mut int_params, &mut ptr_params, &mut contains_string_literal);

        if contains_string_literal {
            for p in params {
                ptr_params.insert(p.clone());
            }
        }

        // === Build parameter types ===
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = params
            .iter()
            .map(|p| {
                if ptr_params.contains(p) {
                    self.context.i8_type().ptr_type(AddressSpace::default()).into()
                } else {
                    self.i32_type.into()
                }
            })
            .collect();

        // === Infer return type recursively ===
fn infer_return_type<'ctx>(
    codegen: &crate::codegen::Codegen<'ctx>,
    expr: &Expr,
    locals: &std::collections::HashSet<String>,
) -> BasicTypeEnum<'ctx> {
    match expr {
        Expr::TypedLiteral { ty, .. } => match ty.as_str() {
            "f32" | "f64" => codegen.context.f32_type().into(),
            "bool" => codegen.context.bool_type().into(),
            "string" | "ptr" => codegen.context.i8_type().ptr_type(AddressSpace::default()).into(),
            _ => codegen.i32_type.into(),
        },
        Expr::BinaryOp { left, right, .. } => {
            let l = infer_return_type(codegen, left, locals);
            let r = infer_return_type(codegen, right, locals);
            if l.is_float_type() || r.is_float_type() {
                codegen.context.f32_type().into()
            } else if l.is_int_type() && r.is_int_type() {
                codegen.i32_type.into()
            } else {
                l
            }
        }
        Expr::Variable(name) => {
            if locals.contains(name) {
                codegen.i32_type.into() // assume int for local vars
            } else {
                codegen.i32_type.into()
            }
        }
        _ => codegen.i32_type.into(),
    }
}

// === Infer function return type ===
let mut inferred_ret_ty = self.i32_type.as_basic_type_enum(); // default
let local_params: std::collections::HashSet<String> = params.iter().cloned().collect();

for stmt in body {
    if let Node::Expr(Expr::Return(Some(inner))) = stmt {
        inferred_ret_ty = infer_return_type(self, inner, &local_params);
    }
}

// === Build function type using inferred return type ===
let fn_ty = inferred_ret_ty.fn_type(&param_types, false);



        // 🪶 Extract TypeDescriptors for dispatch
        println!("🧪 extract_param_type_descriptors for '{}' with params_patterns: {:?}", name, params_patterns);
        let type_descriptors = self.extract_param_type_descriptors(
            params,
            params_patterns,
        );
        println!("🧪 Result: {:?}", type_descriptors);

        let sig = FunctionSignature {
            name: name.clone(),
            param_types: type_descriptors.clone(),
        };

        // 🧩 Generate mangled name using TypeDescriptor
        let mangled_types: Vec<String> = sig.param_types.iter()
            .map(|td| td.to_mangle_string())
            .collect();
        let llvm_name = if mangled_types.is_empty() {
            name.clone()
        } else {
            format!("{}__{}", name, mangled_types.join("_"))
        };

        // 🧠 Skip re-registering if function already exists
        if self.reverse_func_index
            .get(name)
            .map_or(false, |sigs| sigs.iter().any(|s| s.param_types == sig.param_types))
        {
            continue; // Already registered
        }

        let f = self.module.add_function(&llvm_name, fn_ty, None);

        self.functions.insert(sig.clone(), f);
        self.reverse_func_index
            .entry(name.clone())
            .or_default()
            .push(sig.clone());

        // 🎨 Pretty-print type descriptors
        let type_display: Vec<String> = sig.param_types.iter()
            .map(|td| match td {
                crate::ast::types::TypeDescriptor::Entity(name) => name.clone(),
                crate::ast::types::TypeDescriptor::ObjectType(name) => name.clone(),
                crate::ast::types::TypeDescriptor::Primitive(name) => name.clone(),
                crate::ast::types::TypeDescriptor::HttpStatusLiteral(code) => format!("{}", code),
                crate::ast::types::TypeDescriptor::HttpStatusRange(min, _) => format!("{}xx", min / 100),
                crate::ast::types::TypeDescriptor::Any => "_".to_string(),
            })
            .collect();

        println!(
            "🧱 Compiling funcy {} as {}",
            name,
            llvm_name
        );
        println!(
            "🪶 Registered funcy {}({}) as {}",
            name,
            type_display.join(", "),
            llvm_name
        );

    }
}




    // === Compile function bodies ===
    // === Compile function bodies using predeclared signatures ===
for node in nodes {
    if let Node::Expr(Expr::Funcy { name, params, body, is_async, .. }) = node {
        // For each overload signature of this function name
        if let Some(sig_list) = self.reverse_func_index.get(name).cloned() {
    // release immutable borrow immediately by cloning the Vec<FunctionSignature>
    for sig in sig_list {
        let mangled_types: Vec<String> = sig.param_types.iter()
            .map(|td| td.to_mangle_string())
            .collect();
        let llvm_name = if sig.param_types.is_empty() {
            name.clone()
        } else {
            format!("{}__{}", name, mangled_types.join("_"))
        };

        // Skip already-defined functions
        if let Some(f) = self.module.get_function(&llvm_name) {
            if f.count_basic_blocks() > 0 {
                continue;
            }
        }

        let type_display: Vec<String> = sig.param_types.iter()
            .map(|td| match td {
                crate::ast::types::TypeDescriptor::Entity(n) => n.clone(),
                crate::ast::types::TypeDescriptor::ObjectType(n) => n.clone(),
                crate::ast::types::TypeDescriptor::Primitive(n) => n.clone(),
                _ => format!("{:?}", td),
            })
            .collect();

        println!(
            "🧱 Compiling funcy {} as {} ({} overload)",
            name,
            llvm_name,
            type_display.join(", ")
        );

        if *is_async {
            self.compile_async_funcy(name, params, body);
        } else {
            self.compile_funcy(name, params, body, Some(&sig.param_types), None);
        }
    }
}
 else {
            // Fallback — normal single overload
            if *is_async {
                self.compile_async_funcy(name, params, body);
            } else {
                self.compile_funcy(name, params, body, None, None);
            }
        }
    }
}


    // === Detect async entry ===
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

    // === Async bootstrap main() ===
    if let Some(ref entry_name) = async_entry {
        if let Some(fn_val) = self.module.get_function(entry_name) {
            println!("⚡ Injecting async entry `{}` via bootstrap", entry_name);

            let bootstrap = self.module.add_function("main", fn_type, None);
            let boot_block = self.context.append_basic_block(bootstrap, "entry");
            let boot_builder = self.context.create_builder();
            boot_builder.position_at_end(boot_block);

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

            boot_builder
                .build_call(spawn_fn, &[fn_val.as_global_value().as_pointer_value().into()], "spawn_main_task")
                .unwrap();
            boot_builder.build_call(yield_fn, &[], "start_scheduler").unwrap();

            let ret_val = self.i32_type.const_int(0, false);
            boot_builder.build_return(Some(&ret_val)).unwrap();
            return bootstrap;
        }
    }

    // === Compile top-level code ===
    let mut last_int: Option<IntValue> = None;
    for node in nodes {
        println!("🧱 Compiling top-level node: {:?}", node);
        if let Some(v) = self.compile_node(node) {
            if let BasicValueEnum::IntValue(iv) = v {
                last_int = Some(iv);
            }
        }
    }

    // === Generate wrapper main() -> main_async ===
    if self.module.get_function("main").is_none() {
        println!("🔗 Generating wrapper main() -> main_async");

        let wrapper = self.module.add_function("main", fn_type, None);
        let wrap_block = self.context.append_basic_block(wrapper, "entry");
        let wrap_builder = self.context.create_builder();
        wrap_builder.position_at_end(wrap_block);

        let call_site = wrap_builder.build_call(async_fn, &[], "call_main_async").unwrap();
        let result = call_site
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| self.i32_type.const_int(0, false).into());

        wrap_builder.build_return(Some(&result.into_int_value())).unwrap();
    }
    
    // === Ensure main_async ends cleanly ===
// === Ensure main_async ends cleanly ===
let current_block = self.builder.get_insert_block().unwrap();
if current_block.get_terminator().is_none() {
    self.builder.position_at_end(current_block);

    // === Determine actual declared return type ===
    let fn_ret_ty = async_fn.get_type().get_return_type();

    // === Auto join all threads before exit ===
    if let Some(join_all_fn) = self.module.get_function("wpp_thread_join_all") {
        self.builder
            .build_call(join_all_fn, &[], "auto_thread_join_all")
            .unwrap();
    }

    // === Return 0 or null depending on declared type ===
    let ret_val: BasicValueEnum<'ctx> = if let Some(ret_ty) = fn_ret_ty {
    match ret_ty {
        BasicTypeEnum::IntType(i) => i.const_int(0, false).into(),
        BasicTypeEnum::FloatType(f) => f.const_float(0.0).into(),
        BasicTypeEnum::PointerType(p) => p.const_null().into(),
        BasicTypeEnum::ArrayType(_) => self.i32_type.const_int(0, false).into(),
        _ => self.i32_type.const_int(0, false).into(),
    }
} else {
    self.i32_type.const_int(0, false).into()
};


    self.builder.build_return(Some(&ret_val)).unwrap();
    println!("🟢 Added final return terminator to main_async::{:?}", current_block);
}



    async_fn
}









   pub fn run_jit(&self) {
    use std::mem;
    use libc::printf;
    use crate::runtime;
    use inkwell::execution_engine::ExecutionEngine;

    let engine: ExecutionEngine<'_> = self.create_engine();

    unsafe {
        crate::runtime::set_engine(mem::transmute::<
            ExecutionEngine<'_>,
            ExecutionEngine<'static>,
        >(engine.clone()));
    }
    println!("✅ [jit] Execution engine registered globally");

    unsafe {
        // === printf (system) ===
        unsafe extern "C" {
     fn printf(fmt: *const std::os::raw::c_char, ...) -> i32;
}
if let Some(func) = self.module.get_function("printf") {
    let addr = printf as *const () as usize;
    engine.add_global_mapping(&func, addr);
    println!("🔗 [jit] Bound printf @ {:#x}", addr);
} else {
    println!("⚠️ [jit] Missing declaration for printf");
}


        // === Runtime: spawn / yield / return ===
        extern "C" fn wpp_spawn_stub(ptr: *const std::ffi::c_void) {
            runtime::wpp_spawn(ptr as *const ());
        }

        extern "C" fn wpp_yield_stub() {
            runtime::wpp_yield();
        }

        extern "C" fn wpp_return_stub(val: i32) {
    // Allocate a temporary local copy of the value
    let ptr: *const c_void = &val as *const i32 as *const c_void;

    // Tag 1 = integer (see Option A’s type_tag table)
    runtime::wpp_return(ptr, 1);
}

        extern "C" fn wpp_get_last_result_stub() -> i32 {
            runtime::wpp_get_last_result()
        }

        if let Some(f) = self.module.get_function("wpp_spawn") {
            engine.add_global_mapping(&f, wpp_spawn_stub as usize);
        }
        if let Some(f) = self.module.get_function("wpp_yield") {
            engine.add_global_mapping(&f, wpp_yield_stub as usize);
        }
        if let Some(f) = self.module.get_function("wpp_return") {
            engine.add_global_mapping(&f, wpp_return_stub as usize);
        }
        if let Some(f) = self.module.get_function("wpp_get_last_result") {
            engine.add_global_mapping(&f, wpp_get_last_result_stub as usize);
        }

        // === malloc ===
        if let Some(func) = self.module.get_function("malloc") {
            engine.add_global_mapping(&func, libc::malloc as usize);
        }
        
        // === Printing subsystem ===
        // === Printing subsystem (Unified Basic Types) ===
unsafe extern "C" {
    fn wpp_print_value_basic(ptr: *const std::ffi::c_void, type_id: i32);
    fn wpp_print_array(ptr: *const std::ffi::c_void);
    fn wpp_print_object(ptr: *const std::ffi::c_void);
}

for (name, addr) in [
    ("wpp_print_value_basic", wpp_print_value_basic as usize),
    ("wpp_print_array", wpp_print_array as usize),
    ("wpp_print_object", wpp_print_object as usize),
] {
    if let Some(func) = self.module.get_function(name) {
        engine.add_global_mapping(&func, addr);
        println!("🔗 [jit] Bound {}", name);
    } else {
        println!("⚠️ [jit] Missing declaration for {}", name);
    }
}


        // === HTTP subsystem ===
        unsafe extern "C" {
            fn wpp_http_get(ptr: *const std::os::raw::c_char) -> i32;
            fn wpp_http_post(url: *const std::os::raw::c_char, body: *const std::os::raw::c_char) -> i32;
            fn wpp_http_put(url: *const std::os::raw::c_char, body: *const std::os::raw::c_char) -> i32;
            fn wpp_http_patch(url: *const std::os::raw::c_char, body: *const std::os::raw::c_char) -> i32;
            fn wpp_http_delete(url: *const std::os::raw::c_char) -> i32;
            fn wpp_http_status(handle: i32) -> i32;
            fn wpp_http_body(handle: i32) -> *mut std::ffi::c_void;
            fn wpp_http_headers(handle: i32) -> *mut std::ffi::c_void;
            fn wpp_http_free_all();
            fn wpp_register_endpoint(path: *const std::os::raw::c_char, handler: *const ());
            fn wpp_start_server(port: i32);
        }

        let http_funcs = [
            ("wpp_http_get", wpp_http_get as usize),
            ("wpp_http_post", wpp_http_post as usize),
            ("wpp_http_put", wpp_http_put as usize),
            ("wpp_http_patch", wpp_http_patch as usize),
            ("wpp_http_delete", wpp_http_delete as usize),
            ("wpp_http_status", wpp_http_status as usize),
            ("wpp_http_body", wpp_http_body as usize),
            ("wpp_http_headers", wpp_http_headers as usize),
            ("wpp_http_free_all", wpp_http_free_all as usize),
            ("wpp_register_endpoint", wpp_register_endpoint as usize),
            ("wpp_start_server", wpp_start_server as usize),
        ];

        for (name, addr) in http_funcs {
            if let Some(func) = self.module.get_function(name) {
                engine.add_global_mapping(&func, addr);
                println!("🔗 [jit] Bound {}", name);
            } else {
                println!("⚠️ [jit] Missing declaration for {}", name);
            }
        }
       unsafe extern "C" {
    fn wpp_thread_spawn_gc(ptr: *const std::ffi::c_void) -> *mut std::ffi::c_void;
    fn wpp_thread_join(ptr: *mut std::ffi::c_void);
    fn wpp_thread_poll(ptr: *mut std::ffi::c_void) -> i32;
    fn wpp_thread_state_new(initial: i32) -> *mut std::ffi::c_void;
    fn wpp_thread_state_get(ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn wpp_thread_state_set(ptr: *mut std::ffi::c_void, val: i32);
}

for (name, addr) in [
    ("wpp_thread_spawn_gc", wpp_thread_spawn_gc as usize),
    ("wpp_thread_join", wpp_thread_join as usize),
    ("wpp_thread_poll", wpp_thread_poll as usize),
    ("wpp_thread_state_new", wpp_thread_state_new as usize),
    ("wpp_thread_state_get", wpp_thread_state_get as usize),
    ("wpp_thread_state_set", wpp_thread_state_set as usize),
] {
    if let Some(func) = self.module.get_function(name) {
        engine.add_global_mapping(&func, addr);
        println!("🔗 [jit] Bound {}", name);
    } else {
        println!("⚠️ [jit] Missing declaration for {}", name);
    }
}
unsafe extern "C" {
    fn wpp_thread_join_all();
}
if let Some(func) = self.module.get_function("wpp_thread_join_all") {
    engine.add_global_mapping(&func, wpp_thread_join_all as usize);
    println!("🔗 [jit] Bound wpp_thread_join_all");
}
unsafe extern "C" {
    fn wpp_mutex_new(initial: i32) -> *mut std::ffi::c_void;
    fn wpp_mutex_lock(ptr: *mut std::ffi::c_void, thread_id: i32);
    fn wpp_mutex_unlock(ptr: *mut std::ffi::c_void);
}

for (name, addr) in [
    ("wpp_mutex_new", wpp_mutex_new as usize),
    ("wpp_mutex_lock", wpp_mutex_lock as usize),
    ("wpp_mutex_unlock", wpp_mutex_unlock as usize),
] {
    if let Some(func) = self.module.get_function(name) {
        engine.add_global_mapping(&func, addr);
        println!("🔗 [jit] Bound {}", name);
    } else {
        println!("⚠️ [jit] Missing declaration for {}", name);
    }
}
// === String subsystem ===
unsafe extern "C" {
    fn wpp_str_concat(a: *const std::os::raw::c_char, b: *const std::os::raw::c_char) -> *mut std::os::raw::c_char;
}

if let Some(func) = self.module.get_function("wpp_str_concat") {
    engine.add_global_mapping(&func, wpp_str_concat as usize);
    println!("🔗 [jit] Bound wpp_str_concat");
} else {
    println!("⚠️ [jit] Missing declaration for wpp_str_concat");
}



        // === runtime_wait ===
        unsafe extern "C" {
            fn wpp_runtime_wait();
        }
        if let Some(func) = self.module.get_function("wpp_runtime_wait") {
            engine.add_global_mapping(&func, wpp_runtime_wait as usize);
        }
        

    }
    // === Link cross-module imports ===
if let (Some(wms_arc), Some(resolver_arc)) = (&self.wms, &self.resolver) {
    println!("🧩 Linking runtime imports across modules...");

    // 🔒 Lock both Arc<Mutex<T>> values
    let wms = wms_arc.lock().unwrap();
    let mut resolver = resolver_arc.lock().unwrap();

    // ✅ Now we can call methods on the real ExportResolver
    resolver.link_imports_runtime(&engine, &self.module, &wms);
}

    // === Link Rust modules (FFI) ===
    println!("🧩 Linking Rust modules into JIT context...");
    if let Some(wms_arc) = &self.wms {
        let wms = wms_arc.lock().unwrap();
        if let Err(e) = crate::runtime::link_rust_modules(&engine, &self.module, &wms) {
            eprintln!("⚠️ [jit] Failed to link Rust modules: {}", e);
        } else {
            println!("✅ [jit] Rust modules linked successfully.");
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

pub fn compile_new_instance(
    &mut self,
    entity: &str,
    args: &Vec<Expr>,
) -> PointerValue<'ctx> {
    println!("🏗️ [new_instance] Allocating entity '{}'", entity);

    // === 1️⃣ Get the entity definition ===
    let entity_info = self
        .entities
        .get(entity)
        .unwrap_or_else(|| panic!("Unknown entity type: {}", entity));

    let struct_ty = entity_info.struct_type;

    // === 2️⃣ Allocate memory for the instance ===
    let typed_ptr = self
        .builder
        .build_alloca(struct_ty, &format!("{}_ptr", entity))
        .unwrap();

    // === 3️⃣ Optional: call constructor if it exists (e.g. Dog.new) ===
    let ctor_sig = FunctionSignature {
        name: format!("{}.new", entity),
        param_types: args.iter().map(|_| TypeDescriptor::Primitive("i32".to_string())).collect(),
    };

    if let Some(func) = self.functions.get(&ctor_sig).cloned() {
        println!("🧩 [new_instance] Calling constructor '{}.new'", entity);

        let mut compiled_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();

        // Pass 'me' argument (the struct pointer itself)
        compiled_args.push(typed_ptr.into());

        // Compile constructor arguments
        for a in args {
            compiled_args.push(self.compile_expr(a).into());
        }

        // Call the constructor
        self.builder
            .build_call(func, &compiled_args, &format!("call_ctor_{}", entity))
            .unwrap();
    } else {
        println!("⚠️ [new_instance] No constructor found for '{}', skipping init", entity);
    }

    // === 4️⃣ Return pointer to the instance ===
    typed_ptr
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
    param_override: Option<&[TypeDescriptor]>,
    entity_name: Option<&str>, // 👈 added
 // 👈 optional explicit type list
) -> FunctionValue<'ctx> {
    // === Step 1: Detect inferred types from body ===
    let mut int_params = std::collections::HashSet::new();
    let mut ptr_params = std::collections::HashSet::new();
    let mut float_params = std::collections::HashSet::new();
    let mut bool_params = std::collections::HashSet::new();
    let mut contains_string_literal = false;

    fn scan_for_types(
    nodes: &[Node],
    int_params: &mut std::collections::HashSet<String>,
    ptr_params: &mut std::collections::HashSet<String>,
    float_params: &mut std::collections::HashSet<String>,
    bool_params: &mut std::collections::HashSet<String>,
    contains_string_literal: &mut bool,
) {
    for node in nodes {
        match node {
            Node::Expr(expr) => match expr {
                Expr::BinaryOp { left, right, op } => {
                    // 🧵 detect string concatenation
                    if let (Expr::StringLiteral(_), _) | (_, Expr::StringLiteral(_)) =
                        (&**left, &**right)
                    {
                        *contains_string_literal = true;
                        if let Expr::Variable(name) = left.as_ref() {
                            ptr_params.insert(name.clone());
                        }
                        if let Expr::Variable(name) = right.as_ref() {
                            ptr_params.insert(name.clone());
                        }
                    }

                    // 🧮 detect arithmetic or logical types
                    else if ["+", "-", "*", "/", "%"].contains(&op.as_str()) {
                        // detect if either side is float literal
                        let mut is_float = false;
                        if let Expr::TypedLiteral { ty, .. } = &**left {
                            if ty == "f32" {
                                is_float = true;
                            }
                        }
                        if let Expr::TypedLiteral { ty, .. } = &**right {
                            if ty == "f32" {
                                is_float = true;
                            }
                        }

                        if is_float {
                            if let Expr::Variable(name) = left.as_ref() {
                                float_params.insert(name.clone());
                            }
                            if let Expr::Variable(name) = right.as_ref() {
                                float_params.insert(name.clone());
                            }
                        } else {
                            if let Expr::Variable(name) = left.as_ref() {
                                int_params.insert(name.clone());
                            }
                            if let Expr::Variable(name) = right.as_ref() {
                                int_params.insert(name.clone());
                            }
                        }
                    }

                    // 🔀 detect boolean expressions (comparisons, logic ops)
                    else if ["==", "!=", "<", ">", "<=", ">=", "and", "or"].contains(&op.as_str()) {
                        if let Expr::Variable(name) = left.as_ref() {
                            bool_params.insert(name.clone());
                        }
                        if let Expr::Variable(name) = right.as_ref() {
                            bool_params.insert(name.clone());
                        }
                    }

                    // recursive descent into both sides
                    scan_for_types(
                        &[Node::Expr(*left.clone())],
                        int_params,
                        ptr_params,
                        float_params,
                        bool_params,
                        contains_string_literal,
                    );
                    scan_for_types(
                        &[Node::Expr(*right.clone())],
                        int_params,
                        ptr_params,
                        float_params,
                        bool_params,
                        contains_string_literal,
                    );
                }

                Expr::StringLiteral(_) => *contains_string_literal = true,

                Expr::TypedLiteral { ty, .. } => {
                    match ty.as_str() {
                        "f32" => {
                            float_params.insert("".to_string());
                        }
                        "bool" => {
                            bool_params.insert("".to_string());
                        }
                        _ => {}
                    }
                }

                Expr::Call { args, .. } => {
                    for a in args {
                        if let Expr::StringLiteral(_) = a {
                            *contains_string_literal = true;
                        }
                        scan_for_types(
                            &[Node::Expr(a.clone())],
                            int_params,
                            ptr_params,
                            float_params,
                            bool_params,
                            contains_string_literal,
                        );
                    }
                }

                Expr::Return(inner) => {
                    if let Some(inner_expr) = inner {
                        scan_for_types(
                            &[Node::Expr(*inner_expr.clone())],
                            int_params,
                            ptr_params,
                            float_params,
                            bool_params,
                            contains_string_literal,
                        );
                    }
                }

                Expr::If { cond, then_branch, else_branch } => {
                    scan_for_types(
                        &[Node::Expr(*cond.clone())],
                        int_params,
                        ptr_params,
                        float_params,
                        bool_params,
                        contains_string_literal,
                    );
                    scan_for_types(
                        then_branch,
                        int_params,
                        ptr_params,
                        float_params,
                        bool_params,
                        contains_string_literal,
                    );
                    if let Some(e) = else_branch {
                        scan_for_types(
                            e,
                            int_params,
                            ptr_params,
                            float_params,
                            bool_params,
                            contains_string_literal,
                        );
                    }
                }

                Expr::While { cond, body } => {
                    scan_for_types(
                        &[Node::Expr(*cond.clone())],
                        int_params,
                        ptr_params,
                        float_params,
                        bool_params,
                        contains_string_literal,
                    );
                    scan_for_types(
                        body,
                        int_params,
                        ptr_params,
                        float_params,
                        bool_params,
                        contains_string_literal,
                    );
                }

                Expr::Funcy { body, .. } => {
                    scan_for_types(
                        body,
                        int_params,
                        ptr_params,
                        float_params,
                        bool_params,
                        contains_string_literal,
                    );
                }

                _ => {}
            },
            _ => {}
        }
    }
}


    scan_for_types(
        body,
        &mut int_params,
        &mut ptr_params,
        &mut float_params,
        &mut bool_params,
        &mut contains_string_literal,
    );

    // === Step 2: Handle string fallback ===
    if contains_string_literal && int_params.is_empty() && float_params.is_empty() {
        for p in params {
            ptr_params.insert(p.clone());
        }
    }
    // === 🧩 OOPSIE: inject implicit `me` for entity methods ===
let mut effective_params = params.to_vec();
if let Some(ent_name) = entity_name {
    // prepend "me:ptr" if it's not already there
    if !effective_params.iter().any(|p| p.starts_with("me")) {
        effective_params.insert(0, "me:ptr".to_string());
    }
    println!("🧩 Added implicit 'me' to method of entity '{}'", ent_name);
}
    // === Step 3: Build LLVM parameter types (explicit-type aware) ===
let mut param_type_names: Vec<String> = Vec::new();

let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = effective_params
    .iter()
    .map(|p| {
        // Handle explicit annotations like "a:f32"
        let (pname, pty) = if let Some((n, t)) = p.split_once(':') {
            (n.to_string(), t.to_string())
        } else {
            (p.clone(), "i32".to_string())
        };

        param_type_names.push(pty.clone());

        match pty.as_str() {
            "i32" | "int" => self.i32_type.into(),
            "f32" | "float" | "f64" => self.context.f32_type().into(),
            "bool" => self.context.bool_type().into(),
            "ptr" | "string" => self.context.i8_type().ptr_type(AddressSpace::default()).into(),
            _ => {
                // fallback based on inferred type sets
                if ptr_params.contains(&pname) {
                    self.context.i8_type().ptr_type(AddressSpace::default()).into()
                } else if float_params.contains(&pname) {
                    self.context.f32_type().into()
                } else if bool_params.contains(&pname) {
                    self.context.bool_type().into()
                } else {
                    self.i32_type.into()
                }
            }
        }
    })
    .collect();
    

// === Step 4: Apply override if present ===
let final_param_types = if let Some(overrides) = param_override {
    param_type_names = overrides.iter().map(|td| td.to_mangle_string()).collect();
    // Rebuild LLVM param_types based on TypeDescriptors
    overrides.iter().map(|td| {
        match td {
            TypeDescriptor::Entity(_) | TypeDescriptor::ObjectType(_) => {
                // Entities and objects are passed as pointers
                self.context.i8_type().ptr_type(AddressSpace::default()).into()
            }
            TypeDescriptor::Primitive(name) => {
                match name.as_str() {
                    "i32" | "int" => self.i32_type.into(),
                    "f32" | "float" | "f64" => self.context.f32_type().into(),
                    "bool" => self.context.bool_type().into(),
                    "ptr" | "string" => self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                    _ => self.i32_type.into(),
                }
            }
            _ => self.i32_type.into(), // Default for HttpStatus, Any, etc.
        }
    }).collect()
} else {
    param_types
};

    let llvm_name = if let Some(ent_name) = entity_name {
    format!("{}.{}", ent_name, name)  // e.g. Dog.bark
} else if param_type_names.is_empty() {
    name.to_string()
} else {
    format!("{}__{}", name, param_type_names.join("_"))
};


    // === Step 5: Create or fetch LLVM function ===
    // === 🧮 Step 5: Create correct LLVM function type ===
let fn_type = if name == "add" && param_type_names.iter().all(|t| t == "ptr") {
    // 🧠 String overload returns a pointer
    self.context
        .i8_type()
        .ptr_type(AddressSpace::default())
        .fn_type(&final_param_types, false)
} else if param_type_names.iter().any(|t| t == "f32" || t == "f64" || t == "float") {
    // 🧮 Float overload returns float
    self.context.f32_type().fn_type(&final_param_types, false)
} else if param_type_names.iter().any(|t| t == "bool") {
    // 🧩 Bool overload returns i1
    self.context.bool_type().fn_type(&final_param_types, false)
} else {
    // 🧱 Default integer return
    self.i32_type.fn_type(&final_param_types, false)
};


    let function = if let Some(existing) = self.module.get_function(&llvm_name) {
        existing
    } else {
        self.module.add_function(&llvm_name, fn_type, None)
    };

    println!("🧱 Compiling funcy {} as {}", name, llvm_name);

    // === Step 6: Create entry block ===
    let entry = self.context.append_basic_block(function, "entry");
    let saved_block = self.builder.get_insert_block();
    self.builder.position_at_end(entry);

    // === Step 7: Allocate locals (type-accurate) ===
    let mut local_vars: HashMap<String, VarInfo<'ctx>> = HashMap::new();
    for (i, param_name) in effective_params.iter().enumerate() {
    let param = function.get_nth_param(i as u32).unwrap();
    let param_ty = param.get_type();

    // 🧠 Split "a:f32" -> ("a", "f32")
    let pure_name = if let Some((n, _)) = param_name.split_once(':') {
        n.to_string()
    } else {
        param_name.clone()
    };

    let alloca = self.builder.build_alloca(param_ty, &pure_name).unwrap();
    self.builder.build_store(alloca, param).unwrap();

    local_vars.insert(
        pure_name,
        VarInfo {
            ptr: alloca,
            ty: param_ty,
            is_const: false,
            is_thread_state: false,
            entity_type: None,
            object_type_name: None,
        },
    );
}


    // === Step 8: Replace current scope ===
    let old_vars = std::mem::replace(&mut self.vars, local_vars);

    // === Step 9: Compile body ===
    let mut last_val: Option<BasicValueEnum<'ctx>> = None;
    for node in body {
        last_val = self.compile_node(node);
    }

   // === Step 10: Type-aware return handling with bool coercion ===
if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
    let fn_ret_ty = function.get_type().get_return_type();

    // Determine correct return value
    let ret_val: BasicValueEnum<'ctx> = if let Some(val) = last_val {
        if let Some(rt) = fn_ret_ty {
            match rt {
                // int → float
                BasicTypeEnum::FloatType(f) if val.is_int_value() => {
                    let casted = self
                        .builder
                        .build_signed_int_to_float(val.into_int_value(), f, "int_to_float_ret")
                        .unwrap();
                    casted.into()
                }
                // float → int
                BasicTypeEnum::IntType(i) if val.is_float_value() => {
                    let casted = self
                        .builder
                        .build_float_to_signed_int(val.into_float_value(), i, "float_to_int_ret")
                        .unwrap();
                    casted.into()
                }
                // i32 ↔ i1 (for bool returns or accidental widening)
                BasicTypeEnum::IntType(i) if val.is_int_value() => {
                    let src = val.into_int_value();
                    let src_bits = src.get_type().get_bit_width();
                    let dst_bits = i.get_bit_width();
                    let adjusted = if src_bits > dst_bits {
                        self.builder
                            .build_int_truncate(src, i, "ret_trunc_i32_to_i1")
                            .unwrap()
                    } else if src_bits < dst_bits {
                        self.builder
                            .build_int_z_extend(src, i, "ret_zext_i1_to_i32")
                            .unwrap()
                    } else {
                        src
                    };
                    adjusted.into()
                }
                // match already OK
                _ => val,
            }
        } else {
            val
        }
    } else {
        // default 0/null
        if let Some(rt) = fn_ret_ty {
            match rt {
                BasicTypeEnum::IntType(i) => i.const_int(0, false).into(),
                BasicTypeEnum::FloatType(f) => f.const_float(0.0).into(),
                BasicTypeEnum::PointerType(p) => p.const_null().into(),
                _ => self.i32_type.const_int(0, false).into(),
            }
        } else {
            self.i32_type.const_int(0, false).into()
        }
    };

    self.builder.build_return(Some(&ret_val)).unwrap();
}



    // === Step 11: Restore previous state ===
    self.vars = old_vars;
    if let Some(block) = saved_block {
        self.builder.position_at_end(block);
    }

    // === Step 12: Register overload signature ===
    // Convert param_type_names (strings) to TypeDescriptors for registration
    let type_descriptors: Vec<TypeDescriptor> = if let Some(overrides) = param_override {
        overrides.to_vec()
    } else {
        param_type_names.iter().map(|s| TypeDescriptor::Primitive(s.clone())).collect()
    };

    let sig = FunctionSignature {
        name: name.to_string(),
        param_types: type_descriptors,
    };
    self.functions.insert(sig.clone(), function);
    self.reverse_func_index
        .entry(name.to_string())
        .or_default()
        .push(sig.clone());

    let type_display: Vec<String> = sig.param_types.iter()
        .map(|td| td.to_mangle_string())
        .collect();
    println!(
        "🪶 Registered funcy {}({}) as {}",
        name,
        type_display.join(", "),
        llvm_name
    );

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
        is_thread_state: false,
        entity_type: None,
        object_type_name: None,
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
    let sig = FunctionSignature {
    name: name.to_string(),
    param_types: params.iter().map(|_| TypeDescriptor::Primitive("i32".to_string())).collect(),
};

self.functions.insert(sig.clone(), function);
self.reverse_func_index
    .entry(name.to_string())
    .or_default()
    .push(sig);


    println!("✅ async funcy {} compiled successfully", name);
    function
}



pub fn compile_entity(&mut self, entity: &EntityNode) {
        println!("🏗️ Compiling entity: {}", entity.name);

        // === 1️⃣ Inherit base fields (if any) ===
        let mut all_fields: Vec<(String, BasicTypeEnum<'ctx>)> = Vec::new();

        if let Some(base_name) = &entity.base {
            if let Some(base_entity) = self.entities.get(base_name) {
                println!("🔗 Inheriting fields from base entity '{}'", base_name);
                all_fields.extend(base_entity.fields.clone());
            } else {
                eprintln!("⚠️ Base entity '{}' not found", base_name);
            }
        }

        // === 2️⃣ Add this entity’s fields ===
        for member in &entity.members {
            if let EntityMember::Field { name, .. } = member {
                all_fields.push((name.clone(), self.i32_type.into()));
            }
        }

        // === 3️⃣ Define LLVM struct type for the entity ===
        let struct_type = self.context.opaque_struct_type(&entity.name);
        let field_types: Vec<_> = all_fields.iter().map(|(_, t)| *t).collect();
        struct_type.set_body(&field_types, false);

        // === 4️⃣ Register methods ===
        let mut methods = HashMap::new();

        for member in &entity.members {
            if let EntityMember::Method { name, func } = member {
                if let Expr::Funcy {
                    params,
                    body,
                    is_async,
                    ..
                } = func
                {
                    // 👇 compile_funcy with entity context ("Dog.bark")
                    let full_name = format!("{}.{}", entity.name, name);

// Compile with qualified name
let func_val: FunctionValue<'_> = self.compile_funcy(
    &full_name,              // ✅ use "Dog.bark" instead of "bark"
    params,
    body,
    None,
    Some(&entity.name),
);

// Register under both function table and entity-local map
self.functions.insert(
    FunctionSignature { name: full_name.clone(), param_types: vec![] },
    func_val,
);

methods.insert(name.clone(), func_val);

                }
            }
        }

        // === 5️⃣ Register the entity in Codegen context ===
        let oopsie = OopsieEntity {
            name: entity.name.clone(),
            base: entity.base.clone(),
            struct_type,
            fields: all_fields,
            methods,
        };

        self.entities.insert(entity.name.clone(), oopsie);
        println!("✅ Entity '{}' compiled successfully", entity.name);
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



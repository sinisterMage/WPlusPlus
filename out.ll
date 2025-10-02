; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_exception_flag = global i1 false
@_exception_value_i32 = global i32 0
@_exception_value_str = global ptr null
@strlit_0 = private constant [16 x i8] c"hello from W++!\00"
@fmt_s = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

define i32 @main() {
entry:
  %exc_flag = alloca i32, align 4
  store i32 0, ptr %exc_flag, align 4
  %exc_val_i32 = alloca i32, align 4
  store i32 0, ptr %exc_val_i32, align 4
  %exc_val_str = alloca ptr, align 8
  store ptr null, ptr %exc_val_str, align 8
  %call_printf_str = call i32 (ptr, ...) @printf(ptr @fmt_s, ptr @strlit_0)
  ret i32 0
}

declare i32 @printf(ptr, ...)

declare i32 @puts(ptr)

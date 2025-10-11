; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_exception_flag = global i1 false
@_exception_value_i32 = global i32 0
@_exception_value_str = global ptr null

define i32 @main_async() {
entry:
  %exc_flag = alloca i32, align 4
  store i32 0, ptr %exc_flag, align 4
  %exc_val_i32 = alloca i32, align 4
  store i32 0, ptr %exc_val_i32, align 4
  %exc_val_str = alloca ptr, align 8
  store ptr null, ptr %exc_val_str, align 8
  %a = alloca i32, align 4
  store i32 10, ptr %a, align 4
  %b = alloca i64, align 8
  store i64 1410065407, ptr %b, align 4
  %c = alloca double, align 8
  store double 3.140000e+00, ptr %c, align 8
  %d = alloca double, align 8
  %load_c = load double, ptr %c, align 8
  %fadd = fadd double %load_c, 2.500000e+00
  store double %fadd, ptr %d, align 8
  %small = alloca i8, align 1
  store i8 5, ptr %small, align 1
  %z = alloca i32, align 4
  %load_small = load i8, ptr %small, align 1
  %zext = zext i8 %load_small to i32
  store i32 %zext, ptr %z, align 4
  store i32 42, ptr %a, align 4
  store double 9.810000e+00, ptr %c, align 8
  ret i32 0
}

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async

after_return:                                     ; No predecessors!
  ret i32 0
}

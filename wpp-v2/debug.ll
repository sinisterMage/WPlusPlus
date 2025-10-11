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
  %big = alloca i32, align 4
  store i32 1410065407, ptr %big, align 4
  %c = alloca double, align 8
  store double 3.140000e+00, ptr %c, align 8
  %tiny = alloca i32, align 4
  store i32 5, ptr %tiny, align 4
  %flag = alloca i1, align 1
  store i1 true, ptr %flag, align 1
  %z = alloca i32, align 4
  %load_big = load i32, ptr %big, align 4
  store i32 %load_big, ptr %z, align 4
  ret i32 0
}

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async

after_return:                                     ; No predecessors!
  ret i32 0
}

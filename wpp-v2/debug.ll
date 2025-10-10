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
  ret i32 0
}

define i32 @__wpp_dummy() {
safe_after_main:
  ret i32 0
}

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async

after_return:                                     ; No predecessors!
  ret i32 0
}

; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_exception_flag = global i1 false
@_exception_value_i32 = global i32 0
@_exception_value_str = global ptr null
@strlit_0 = private constant [9 x i8] c"starting\00"
@fmt_s = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@strlit_1 = private constant [5 x i8] c"done\00"
@fmt_s.1 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@strlit_2 = private constant [11 x i8] c"waiting...\00"
@fmt_s.2 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

define i32 @main() {
entry:
  %exc_flag = alloca i32, align 4
  store i32 0, ptr %exc_flag, align 4
  %exc_val_i32 = alloca i32, align 4
  store i32 0, ptr %exc_val_i32, align 4
  %exc_val_str = alloca ptr, align 8
  store ptr null, ptr %exc_val_str, align 8
  call void @wpp_spawn(ptr @task)
  call void @wpp_yield()
  ret i32 0
}

define i32 @wait() {
entry:
  %call_printf_str = call i32 (ptr, ...) @printf(ptr @fmt_s.2, ptr @strlit_2)
  ret i32 0
}

define i32 @task() {
entry:
  %call_printf_str = call i32 (ptr, ...) @printf(ptr @fmt_s, ptr @strlit_0)
  %call_wait = call i32 @wait()
  call void @wpp_yield()
  %call_printf_str1 = call i32 (ptr, ...) @printf(ptr @fmt_s.1, ptr @strlit_1)
  call void @wpp_yield()
  ret i32 0
}

declare i32 @printf(ptr, ...)

declare i32 @puts(ptr)

declare void @wpp_yield()

declare void @wpp_spawn(ptr)

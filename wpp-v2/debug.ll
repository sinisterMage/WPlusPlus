; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_exception_flag = global i1 false
@_exception_value_i32 = global i32 0
@_exception_value_str = global ptr null
@strlit_0 = private constant [15 x i8] c"calculating...\00"
@fmt_s = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@strlit_1 = private constant [11 x i8] c"waiting...\00"
@fmt_s.1 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@strlit_2 = private constant [8 x i8] c"result:\00"
@fmt_s.2 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@fmt_d = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

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

define i32 @wait() {
entry:
  %call_printf_str = call i32 (ptr, ...) @printf(ptr @fmt_s.1, ptr @strlit_1)
  ret i32 0
}

define i32 @calc() {
entry:
  %call_printf_str = call i32 (ptr, ...) @printf(ptr @fmt_s, ptr @strlit_0)
  %call_wait = call i32 @wait()
  call void @wpp_yield()
  %await_result = call i32 @wpp_get_last_result()
  call void @wpp_return(i32 42)
  ret i32 42

after_return:                                     ; No predecessors!
  call void @wpp_return(i32 0)
  call void @wpp_yield()
  ret i32 0
}

declare i32 @printf(ptr, ...)

declare i32 @puts(ptr)

declare void @wpp_yield()

declare i32 @wpp_get_last_result()

declare void @wpp_return(i32)

define i32 @main() {
entry:
  %call_calc = call i32 @calc()
  call void @wpp_yield()
  %await_result = call i32 @wpp_get_last_result()
  %x = alloca i32, align 4
  store i32 %await_result, ptr %x, align 4
  %call_printf_str = call i32 (ptr, ...) @printf(ptr @fmt_s.2, ptr @strlit_2)
  %load_x = load i32, ptr %x, align 4
  %call_printf_int = call i32 (ptr, ...) @printf(ptr @fmt_d, i32 %load_x)
  call void @wpp_return(i32 0)
  call void @wpp_yield()
  ret i32 0
}

define i32 @main.3() {
entry:
  call void @wpp_spawn(ptr @main)
  call void @wpp_yield()
  ret i32 0
}

declare void @wpp_spawn(ptr)

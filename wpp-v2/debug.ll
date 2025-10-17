; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@ret_fmt = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@strlit_0 = private constant [19 x i8] c"string add called!\00"
@strlit_1 = private constant [3 x i8] c"hi\00"
@strlit_2 = private constant [6 x i8] c"there\00"

declare void @wpp_print_value(ptr, i32)

declare void @wpp_print_array(ptr)

declare void @wpp_print_object(ptr)

declare ptr @wpp_str_concat(ptr, ptr)

declare i32 @wpp_http_get(ptr)

declare void @wpp_register_endpoint(ptr, ptr)

declare void @wpp_start_server(i32)

declare ptr @wpp_thread_spawn_gc(ptr)

declare void @wpp_thread_join(ptr)

declare i32 @wpp_thread_poll(ptr)

declare ptr @wpp_thread_state_new(i32)

declare ptr @wpp_thread_state_get(ptr)

declare void @wpp_thread_state_set(ptr, i32)

declare void @wpp_thread_join_all()

declare ptr @wpp_mutex_new(i32)

declare void @wpp_mutex_lock(ptr, i32)

declare void @wpp_mutex_unlock(ptr)

define i32 @main_async() {
entry:
  %exc_flag = alloca i32, align 4
  store i32 0, ptr %exc_flag, align 4
  %exc_val_i32 = alloca i32, align 4
  store i32 0, ptr %exc_val_i32, align 4
  %exc_val_str = alloca ptr, align 8
  store ptr null, ptr %exc_val_str, align 8
  %call_add = call i32 @add__i32_i32(i32 2, i32 3)
  call void @wpp_print_i32(i32 %call_add)
  %call_add1 = call i32 @add__ptr_ptr(ptr @strlit_1, ptr @strlit_2)
  call void @wpp_print_i32(i32 %call_add1)
  call void @wpp_thread_join_all()
  ret i32 0
}

define i32 @add__i32_i32(i32 %0, i32 %1) {
entry:
  %a = alloca i32, align 4
  store i32 %0, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %1, ptr %b, align 4
  %load_a = load i32, ptr %a, align 4
  %load_b = load i32, ptr %b, align 4
  %addtmp = add i32 %load_a, %load_b
  call void @wpp_return(i32 %addtmp)
  ret i32 %addtmp

after_return:                                     ; No predecessors!
  ret i32 0

entry1:                                           ; No predecessors!
  %a2 = alloca i32, align 4
  store i32 %0, ptr %a2, align 4
  %b3 = alloca i32, align 4
  store i32 %1, ptr %b3, align 4
  %load_a4 = load i32, ptr %a2, align 4
  %load_b5 = load i32, ptr %b3, align 4
  %addtmp6 = add i32 %load_a4, %load_b5
  call void @wpp_return(i32 %addtmp6)
  ret i32 %addtmp6

after_return7:                                    ; No predecessors!
  ret i32 0

entry8:                                           ; No predecessors!
  %a9 = alloca i32, align 4
  store i32 %0, ptr %a9, align 4
  %b10 = alloca i32, align 4
  store i32 %1, ptr %b10, align 4
  call void @wpp_print_value(ptr @strlit_0, i32 0)
  %load_a11 = load i32, ptr %a9, align 4
  %load_b12 = load i32, ptr %b10, align 4
  %addtmp13 = add i32 %load_a11, %load_b12
  call void @wpp_return(i32 %addtmp13)
  ret i32 %addtmp13

after_return14:                                   ; No predecessors!
  ret i32 0
}

define i32 @add__ptr_ptr(ptr %0, ptr %1) {
entry:
  %a = alloca ptr, align 8
  store ptr %0, ptr %a, align 8
  %b = alloca ptr, align 8
  store ptr %1, ptr %b, align 8
  %load_a = load ptr, ptr %a, align 8
  %load_b = load ptr, ptr %b, align 8
  %concat = call ptr @wpp_str_concat(ptr %load_a, ptr %load_b)
  %print_return = call i32 (ptr, ...) @printf(ptr @ret_fmt, ptr %concat)
  call void @wpp_return(i32 0)
  ret i32 0

after_return:                                     ; No predecessors!
  ret i32 0
}

declare void @wpp_return(i32)

declare i32 @printf(ptr, ...)

declare void @wpp_print_i32(i32)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async
}

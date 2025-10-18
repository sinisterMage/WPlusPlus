; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null

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
  %tmp_int = alloca i32, align 4
  store i32 %call_add, ptr %tmp_int, align 4
  call void @wpp_print_value_basic(ptr %tmp_int, i32 1)
  %call_add1 = call float @add__f32_f32(float 1.500000e+00, float 0x4002666660000000)
  %tmp_float = alloca float, align 4
  store float %call_add1, ptr %tmp_float, align 4
  call void @wpp_print_value_basic(ptr %tmp_float, i32 3)
  %call_add2 = call i32 @add__i32_i32(i32 1, i32 0)
  %tmp_int3 = alloca i32, align 4
  store i32 %call_add2, ptr %tmp_int3, align 4
  call void @wpp_print_value_basic(ptr %tmp_int3, i32 1)
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
  %ret_tmp = alloca i32, align 4
  store i32 %addtmp, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
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
  %ret_tmp7 = alloca i32, align 4
  store i32 %addtmp6, ptr %ret_tmp7, align 4
  call void @wpp_return(ptr %ret_tmp7, i32 1)
  ret i32 %addtmp6

after_return8:                                    ; No predecessors!
  ret i32 0
}

declare void @wpp_return(ptr, i32)

define float @add__f32_f32(float %0, float %1) {
entry:
  %a = alloca float, align 4
  store float %0, ptr %a, align 4
  %b = alloca float, align 4
  store float %1, ptr %b, align 4
  %load_a = load float, ptr %a, align 4
  %load_b = load float, ptr %b, align 4
  %fadd = fadd float %load_a, %load_b
  %ret_tmp = alloca float, align 4
  store float %fadd, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 2)
  ret float %fadd

after_return:                                     ; No predecessors!
  ret float 0.000000e+00
}

define i1 @add__bool_bool(i1 %0, i1 %1) {
entry:
  %a = alloca i1, align 1
  store i1 %0, ptr %a, align 1
  %b = alloca i1, align 1
  store i1 %1, ptr %b, align 1
  %load_a = load i1, ptr %a, align 1
  %load_b = load i1, ptr %b, align 1
  %ortmp = or i1 %load_a, %load_b
  %ret_tmp = alloca i1, align 1
  store i1 %ortmp, ptr %ret_tmp, align 1
  call void @wpp_return(ptr %ret_tmp, i32 3)
  ret i1 %ortmp

after_return:                                     ; No predecessors!
  ret i1 false
}

declare void @wpp_print_value_basic(ptr, i32)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async
}

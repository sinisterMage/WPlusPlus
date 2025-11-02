; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [26 x i8] c"Applying integer function\00"
@newline = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_1 = private constant [38 x i8] c"=== Testing Higher-Order Dispatch ===\00"
@newline.1 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.2 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.3 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_2 = private constant [22 x i8] c"=== Test Complete ===\00"
@newline.4 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1

declare void @wpp_print_value(ptr, i32)

declare void @wpp_print_array(ptr)

declare void @wpp_print_object(ptr)

declare ptr @wpp_str_concat(ptr, ptr)

declare ptr @wpp_readline()

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

define i32 @double__i32(i32 %0) {
entry:
  %x = alloca i32, align 4
  store i32 %0, ptr %x, align 4
  %load_x = load i32, ptr %x, align 4
  %multmp = mul i32 %load_x, 2
  %ret_tmp = alloca i32, align 4
  store i32 %multmp, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %multmp

after_return:                                     ; No predecessors!
  ret i32 0
}

declare void @wpp_return(ptr, i32)

define i32 @triple__i32(i32 %0) {
entry:
  %x = alloca i32, align 4
  store i32 %0, ptr %x, align 4
  %load_x = load i32, ptr %x, align 4
  %multmp = mul i32 %load_x, 3
  %ret_tmp = alloca i32, align 4
  store i32 %multmp, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %multmp

after_return:                                     ; No predecessors!
  ret i32 0
}

define i32 @apply__fn_i32_ret_i32_i32(ptr %0, i32 %1) {
entry:
  %fn = alloca ptr, align 8
  store ptr %0, ptr %fn, align 8
  %data = alloca i32, align 4
  store i32 %1, ptr %data, align 4
  call void @wpp_print_value_basic(ptr @strlit_0, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline)
  %load_fnptr_fn = load ptr, ptr %fn, align 8
  %load_data = load i32, ptr %data, align 4
  %call_indirect_fn = call i32 %load_fnptr_fn(i32 %load_data)
  %ret_tmp = alloca i32, align 4
  store i32 %call_indirect_fn, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %call_indirect_fn

after_return:                                     ; No predecessors!
  ret i32 0
}

declare void @wpp_print_value_basic(ptr, i32)

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  call void @wpp_print_value_basic(ptr @strlit_1, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline.1)
  %result1 = alloca i32, align 4
  %call_apply = call i32 @apply__fn_i32_ret_i32_i32(ptr @double__i32, i32 5)
  store i32 %call_apply, ptr %result1, align 4
  %load_result1 = load i32, ptr %result1, align 4
  %tmp_int = alloca i32, align 4
  store i32 %load_result1, ptr %tmp_int, align 4
  call void @wpp_print_value_basic(ptr %tmp_int, i32 1)
  %print_newline1 = call i32 (ptr, ...) @printf(ptr @newline.2)
  %result2 = alloca i32, align 4
  %call_apply2 = call i32 @apply__fn_i32_ret_i32_i32(ptr @triple__i32, i32 7)
  store i32 %call_apply2, ptr %result2, align 4
  %load_result2 = load i32, ptr %result2, align 4
  %tmp_int3 = alloca i32, align 4
  store i32 %load_result2, ptr %tmp_int3, align 4
  call void @wpp_print_value_basic(ptr %tmp_int3, i32 1)
  %print_newline4 = call i32 (ptr, ...) @printf(ptr @newline.3)
  call void @wpp_print_value_basic(ptr @strlit_2, i32 6)
  %print_newline5 = call i32 (ptr, ...) @printf(ptr @newline.4)
  ret i32 0
}

declare i32 @__submodule_stub()

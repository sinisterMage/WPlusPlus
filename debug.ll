; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [26 x i8] c"Applying func(i32) -> i32\00"
@newline = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_1 = private constant [31 x i8] c"Applying func(i32, i32) -> i32\00"
@newline.1 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_2 = private constant [36 x i8] c"Executing func() -> i32 (no params)\00"
@newline.2 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_3 = private constant [37 x i8] c"Executing func(i32) -> i32 (1 param)\00"
@newline.3 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_4 = private constant [33 x i8] c"Processing single-param function\00"
@newline.4 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_5 = private constant [30 x i8] c"Processing two-param function\00"
@newline.5 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_6 = private constant [24 x i8] c"Composing two functions\00"
@newline.6 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_7 = private constant [44 x i8] c"=== Test 1: Dispatch on Parameter Types ===\00"
@newline.7 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.8 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.9 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_8 = private constant [1 x i8] zeroinitializer
@newline.10 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_9 = private constant [44 x i8] c"=== Test 2: Dispatch on Parameter Count ===\00"
@newline.11 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.12 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.13 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_10 = private constant [1 x i8] zeroinitializer
@newline.14 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_11 = private constant [49 x i8] c"=== Test 3: Complex Multi-Parameter Dispatch ===\00"
@newline.15 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.16 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.17 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_12 = private constant [1 x i8] zeroinitializer
@newline.18 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_13 = private constant [37 x i8] c"=== Test 4: Function Composition ===\00"
@newline.19 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@newline.20 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_14 = private constant [1 x i8] zeroinitializer
@newline.21 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@strlit_15 = private constant [27 x i8] c"=== All Tests Complete ===\00"
@newline.22 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1

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

declare void @wpp_return(ptr, i32)

define i32 @apply__fn_i32_i32_ret_i32_i32_i32(ptr %0, i32 %1, i32 %2) {
entry:
  %fn = alloca ptr, align 8
  store ptr %0, ptr %fn, align 8
  %a = alloca i32, align 4
  store i32 %1, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %2, ptr %b, align 4
  call void @wpp_print_value_basic(ptr @strlit_1, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline.1)
  %load_fnptr_fn = load ptr, ptr %fn, align 8
  %load_a = load i32, ptr %a, align 4
  %load_b = load i32, ptr %b, align 4
  %call_indirect_fn = call i32 %load_fnptr_fn(i32 %load_a, i32 %load_b)
  %ret_tmp = alloca i32, align 4
  store i32 %call_indirect_fn, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %call_indirect_fn

after_return:                                     ; No predecessors!
  ret i32 0
}

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

define i32 @add__i32_i32(i32 %0, i32 %1) {
entry:
  %x = alloca i32, align 4
  store i32 %0, ptr %x, align 4
  %y = alloca i32, align 4
  store i32 %1, ptr %y, align 4
  %load_x = load i32, ptr %x, align 4
  %load_y = load i32, ptr %y, align 4
  %addtmp = add i32 %load_x, %load_y
  %ret_tmp = alloca i32, align 4
  store i32 %addtmp, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %addtmp

after_return:                                     ; No predecessors!
  ret i32 0
}

define i32 @execute__fn__ret_i32(ptr %0) {
entry:
  %fn = alloca ptr, align 8
  store ptr %0, ptr %fn, align 8
  call void @wpp_print_value_basic(ptr @strlit_2, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline.2)
  %load_fnptr_fn = load ptr, ptr %fn, align 8
  %call_indirect_fn = call i32 %load_fnptr_fn()
  %ret_tmp = alloca i32, align 4
  store i32 %call_indirect_fn, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %call_indirect_fn

after_return:                                     ; No predecessors!
  ret i32 0
}

define i32 @execute__fn_i32_ret_i32_i32(ptr %0, i32 %1) {
entry:
  %fn = alloca ptr, align 8
  store ptr %0, ptr %fn, align 8
  %val = alloca i32, align 4
  store i32 %1, ptr %val, align 4
  call void @wpp_print_value_basic(ptr @strlit_3, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline.3)
  %load_fnptr_fn = load ptr, ptr %fn, align 8
  %load_val = load i32, ptr %val, align 4
  %call_indirect_fn = call i32 %load_fnptr_fn(i32 %load_val)
  %ret_tmp = alloca i32, align 4
  store i32 %call_indirect_fn, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %call_indirect_fn

after_return:                                     ; No predecessors!
  ret i32 0
}

define i32 @getFortyTwo() {
entry:
  %ret_tmp = alloca i32, align 4
  store i32 42, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 42

after_return:                                     ; No predecessors!
  ret i32 0
}

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

define i32 @process__fn_i32_ret_i32_i32(ptr %0, i32 %1) {
entry:
  %fn = alloca ptr, align 8
  store ptr %0, ptr %fn, align 8
  %data = alloca i32, align 4
  store i32 %1, ptr %data, align 4
  call void @wpp_print_value_basic(ptr @strlit_4, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline.4)
  %load_fnptr_fn = load ptr, ptr %fn, align 8
  %load_data = load i32, ptr %data, align 4
  %call_indirect_fn = call i32 %load_fnptr_fn(i32 %load_data)
  %addtmp = add i32 %call_indirect_fn, 100
  %ret_tmp = alloca i32, align 4
  store i32 %addtmp, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %addtmp

after_return:                                     ; No predecessors!
  ret i32 0
}

define i32 @process__fn_i32_i32_ret_i32_i32_i32(ptr %0, i32 %1, i32 %2) {
entry:
  %fn = alloca ptr, align 8
  store ptr %0, ptr %fn, align 8
  %a = alloca i32, align 4
  store i32 %1, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 %2, ptr %b, align 4
  call void @wpp_print_value_basic(ptr @strlit_5, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline.5)
  %load_fnptr_fn = load ptr, ptr %fn, align 8
  %load_a = load i32, ptr %a, align 4
  %load_b = load i32, ptr %b, align 4
  %call_indirect_fn = call i32 %load_fnptr_fn(i32 %load_a, i32 %load_b)
  %multmp = mul i32 %call_indirect_fn, 2
  %ret_tmp = alloca i32, align 4
  store i32 %multmp, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %multmp

after_return:                                     ; No predecessors!
  ret i32 0
}

define i32 @multiply__i32_i32(i32 %0, i32 %1) {
entry:
  %x = alloca i32, align 4
  store i32 %0, ptr %x, align 4
  %y = alloca i32, align 4
  store i32 %1, ptr %y, align 4
  %load_x = load i32, ptr %x, align 4
  %load_y = load i32, ptr %y, align 4
  %multmp = mul i32 %load_x, %load_y
  %ret_tmp = alloca i32, align 4
  store i32 %multmp, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %multmp

after_return:                                     ; No predecessors!
  ret i32 0
}

define i32 @compose__fn_i32_ret_i32_fn_i32_ret_i32_i32(ptr %0, ptr %1, i32 %2) {
entry:
  %f = alloca ptr, align 8
  store ptr %0, ptr %f, align 8
  %g = alloca ptr, align 8
  store ptr %1, ptr %g, align 8
  %x = alloca i32, align 4
  store i32 %2, ptr %x, align 4
  call void @wpp_print_value_basic(ptr @strlit_6, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline.6)
  %load_fnptr_f = load ptr, ptr %f, align 8
  %load_fnptr_g = load ptr, ptr %g, align 8
  %load_x = load i32, ptr %x, align 4
  %call_indirect_g = call i32 %load_fnptr_g(i32 %load_x)
  %call_indirect_f = call i32 %load_fnptr_f(i32 %call_indirect_g)
  %ret_tmp = alloca i32, align 4
  store i32 %call_indirect_f, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 %call_indirect_f

after_return:                                     ; No predecessors!
  ret i32 0
}

define i32 @main() {
entry:
  call void @wpp_print_value_basic(ptr @strlit_7, i32 6)
  %print_newline = call i32 (ptr, ...) @printf(ptr @newline.7)
  %r1 = alloca i32, align 4
  %call_apply = call i32 @apply__fn_i32_ret_i32_i32(ptr @double__i32, i32 5)
  store i32 %call_apply, ptr %r1, align 4
  %load_r1 = load i32, ptr %r1, align 4
  %tmp_int = alloca i32, align 4
  store i32 %load_r1, ptr %tmp_int, align 4
  call void @wpp_print_value_basic(ptr %tmp_int, i32 1)
  %print_newline1 = call i32 (ptr, ...) @printf(ptr @newline.8)
  %r2 = alloca i32, align 4
  %call_apply2 = call i32 @apply__fn_i32_i32_ret_i32_i32_i32(ptr @add__i32_i32, i32 3, i32 4)
  store i32 %call_apply2, ptr %r2, align 4
  %load_r2 = load i32, ptr %r2, align 4
  %tmp_int3 = alloca i32, align 4
  store i32 %load_r2, ptr %tmp_int3, align 4
  call void @wpp_print_value_basic(ptr %tmp_int3, i32 1)
  %print_newline4 = call i32 (ptr, ...) @printf(ptr @newline.9)
  call void @wpp_print_value_basic(ptr @strlit_8, i32 6)
  %print_newline5 = call i32 (ptr, ...) @printf(ptr @newline.10)
  call void @wpp_print_value_basic(ptr @strlit_9, i32 6)
  %print_newline6 = call i32 (ptr, ...) @printf(ptr @newline.11)
  %r3 = alloca i32, align 4
  %call_execute = call i32 @execute__fn__ret_i32(ptr @getFortyTwo)
  store i32 %call_execute, ptr %r3, align 4
  %load_r3 = load i32, ptr %r3, align 4
  %tmp_int7 = alloca i32, align 4
  store i32 %load_r3, ptr %tmp_int7, align 4
  call void @wpp_print_value_basic(ptr %tmp_int7, i32 1)
  %print_newline8 = call i32 (ptr, ...) @printf(ptr @newline.12)
  %r4 = alloca i32, align 4
  %call_execute9 = call i32 @execute__fn_i32_ret_i32_i32(ptr @triple__i32, i32 7)
  store i32 %call_execute9, ptr %r4, align 4
  %load_r4 = load i32, ptr %r4, align 4
  %tmp_int10 = alloca i32, align 4
  store i32 %load_r4, ptr %tmp_int10, align 4
  call void @wpp_print_value_basic(ptr %tmp_int10, i32 1)
  %print_newline11 = call i32 (ptr, ...) @printf(ptr @newline.13)
  call void @wpp_print_value_basic(ptr @strlit_10, i32 6)
  %print_newline12 = call i32 (ptr, ...) @printf(ptr @newline.14)
  call void @wpp_print_value_basic(ptr @strlit_11, i32 6)
  %print_newline13 = call i32 (ptr, ...) @printf(ptr @newline.15)
  %r5 = alloca i32, align 4
  %call_process = call i32 @process__fn_i32_ret_i32_i32(ptr @double__i32, i32 10)
  store i32 %call_process, ptr %r5, align 4
  %load_r5 = load i32, ptr %r5, align 4
  %tmp_int14 = alloca i32, align 4
  store i32 %load_r5, ptr %tmp_int14, align 4
  call void @wpp_print_value_basic(ptr %tmp_int14, i32 1)
  %print_newline15 = call i32 (ptr, ...) @printf(ptr @newline.16)
  %r6 = alloca i32, align 4
  %call_process16 = call i32 @process__fn_i32_i32_ret_i32_i32_i32(ptr @multiply__i32_i32, i32 5, i32 6)
  store i32 %call_process16, ptr %r6, align 4
  %load_r6 = load i32, ptr %r6, align 4
  %tmp_int17 = alloca i32, align 4
  store i32 %load_r6, ptr %tmp_int17, align 4
  call void @wpp_print_value_basic(ptr %tmp_int17, i32 1)
  %print_newline18 = call i32 (ptr, ...) @printf(ptr @newline.17)
  call void @wpp_print_value_basic(ptr @strlit_12, i32 6)
  %print_newline19 = call i32 (ptr, ...) @printf(ptr @newline.18)
  call void @wpp_print_value_basic(ptr @strlit_13, i32 6)
  %print_newline20 = call i32 (ptr, ...) @printf(ptr @newline.19)
  %r7 = alloca i32, align 4
  %call_compose = call i32 @compose__fn_i32_ret_i32_fn_i32_ret_i32_i32(ptr @double__i32, ptr @triple__i32, i32 5)
  store i32 %call_compose, ptr %r7, align 4
  %load_r7 = load i32, ptr %r7, align 4
  %tmp_int21 = alloca i32, align 4
  store i32 %load_r7, ptr %tmp_int21, align 4
  call void @wpp_print_value_basic(ptr %tmp_int21, i32 1)
  %print_newline22 = call i32 (ptr, ...) @printf(ptr @newline.20)
  call void @wpp_print_value_basic(ptr @strlit_14, i32 6)
  %print_newline23 = call i32 (ptr, ...) @printf(ptr @newline.21)
  call void @wpp_print_value_basic(ptr @strlit_15, i32 6)
  %print_newline24 = call i32 (ptr, ...) @printf(ptr @newline.22)
  ret i32 0
}

declare i32 @__submodule_stub()

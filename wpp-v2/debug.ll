; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null

declare void @wpp_print_value(ptr, i32)

declare void @wpp_print_array(ptr)

declare void @wpp_print_object(ptr)

declare i32 @wpp_http_get(ptr)

declare void @wpp_register_endpoint(ptr, ptr)

declare void @wpp_start_server(i32)

declare ptr @wpp_thread_spawn_gc(ptr)

declare void @wpp_thread_join(ptr)

declare i32 @wpp_thread_poll(ptr)

declare ptr @wpp_thread_state_new(i32)

declare i32 @wpp_thread_state_get(ptr)

declare void @wpp_thread_state_set(ptr, i32)

define i32 @main_async() {
entry:
  %exc_flag = alloca i32, align 4
  store i32 0, ptr %exc_flag, align 4
  %exc_val_i32 = alloca i32, align 4
  store i32 0, ptr %exc_val_i32, align 4
  %exc_val_str = alloca ptr, align 8
  store ptr null, ptr %exc_val_str, align 8
  %call_thread_spawn_gc = call ptr @wpp_thread_spawn_gc(ptr @worker)
  call void @wpp_thread_join(ptr %call_thread_spawn_gc)
  ret i32 0
}

define i32 @worker() {
entry:
  %counter = alloca ptr, align 8
  %call_thread_state_new = call ptr @wpp_thread_state_new(i32 0)
  store ptr %call_thread_state_new, ptr %counter, align 8
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  br label %while_cond

while_cond:                                       ; preds = %while_body, %entry
  %load_i = load i32, ptr %i, align 4
  %lttmp = icmp slt i32 %load_i, 5
  br i1 %lttmp, label %while_body, label %while_end

while_body:                                       ; preds = %while_cond
  %load_counter = load ptr, ptr %counter, align 8
  %call_thread_state_get = call i32 @wpp_thread_state_get(ptr %load_counter)
  %addtmp = add i32 %call_thread_state_get, 1
  %load_thread_ptr = load ptr, ptr %counter, align 8
  call void @wpp_thread_state_set(ptr %load_thread_ptr, i32 %addtmp)
  %load_i1 = load i32, ptr %i, align 4
  %addtmp2 = add i32 %load_i1, 1
  store i32 %addtmp2, ptr %i, align 4
  br label %while_cond

while_end:                                        ; preds = %while_cond
  br label %after_loop

after_loop:                                       ; preds = %while_end
  %load_counter3 = load ptr, ptr %counter, align 8
  %call_thread_state_get4 = call i32 @wpp_thread_state_get(ptr %load_counter3)
  call void @wpp_print_i32(i32 %call_thread_state_get4)
  ret i32 0

entry5:                                           ; No predecessors!
  %counter6 = alloca ptr, align 8
  %call_thread_state_new7 = call ptr @wpp_thread_state_new(i32 0)
  store ptr %call_thread_state_new7, ptr %counter6, align 8
  %i8 = alloca i32, align 4
  store i32 0, ptr %i8, align 4
  br label %while_cond9

while_cond9:                                      ; preds = %while_body10, %entry5
  %load_i13 = load i32, ptr %i8, align 4
  %lttmp14 = icmp slt i32 %load_i13, 5
  br i1 %lttmp14, label %while_body10, label %while_end11

while_body10:                                     ; preds = %while_cond9
  %load_counter15 = load ptr, ptr %counter6, align 8
  %call_thread_state_get16 = call i32 @wpp_thread_state_get(ptr %load_counter15)
  %addtmp17 = add i32 %call_thread_state_get16, 1
  %load_thread_ptr18 = load ptr, ptr %counter6, align 8
  call void @wpp_thread_state_set(ptr %load_thread_ptr18, i32 %addtmp17)
  %load_i19 = load i32, ptr %i8, align 4
  %addtmp20 = add i32 %load_i19, 1
  store i32 %addtmp20, ptr %i8, align 4
  br label %while_cond9

while_end11:                                      ; preds = %while_cond9
  br label %after_loop12

after_loop12:                                     ; preds = %while_end11
  %load_counter21 = load ptr, ptr %counter6, align 8
  %call_thread_state_get22 = call i32 @wpp_thread_state_get(ptr %load_counter21)
  call void @wpp_print_i32(i32 %call_thread_state_get22)
  ret i32 0
}

declare void @wpp_print_i32(i32)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async
}

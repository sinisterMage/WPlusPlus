; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [25 x i8] c"Running in background...\00"
@strlit_1 = private constant [25 x i8] c"Running in background...\00"

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

declare void @wpp_thread_join_all()

define i32 @main_async() {
entry:
  %exc_flag = alloca i32, align 4
  store i32 0, ptr %exc_flag, align 4
  %exc_val_i32 = alloca i32, align 4
  store i32 0, ptr %exc_val_i32, align 4
  %exc_val_str = alloca ptr, align 8
  store ptr null, ptr %exc_val_str, align 8
  %call_thread_spawn_gc = call ptr @wpp_thread_spawn_gc(ptr @worker)
  br i1 true, label %join_thread, label %cont_thread

join_thread:                                      ; preds = %entry
  call void @wpp_thread_join(ptr %call_thread_spawn_gc)
  br label %cont_thread

cont_thread:                                      ; preds = %join_thread, %entry
  ret i32 0
}

define i32 @worker() {
entry:
  %counter = alloca ptr, align 8
  %call_thread_state_new = call ptr @wpp_thread_state_new(i32 0)
  store ptr %call_thread_state_new, ptr %counter, align 8
  br label %while_cond

while_cond:                                       ; preds = %while_body, %entry
  %load_counter = load ptr, ptr %counter, align 8
  %call_thread_state_get = call i32 @wpp_thread_state_get(ptr %load_counter)
  %lttmp = icmp slt i32 %call_thread_state_get, 3
  br i1 %lttmp, label %while_body, label %while_end

while_body:                                       ; preds = %while_cond
  %load_counter1 = load ptr, ptr %counter, align 8
  %call_thread_state_get2 = call i32 @wpp_thread_state_get(ptr %load_counter1)
  %addtmp = add i32 %call_thread_state_get2, 1
  %load_thread_ptr = load ptr, ptr %counter, align 8
  call void @wpp_thread_state_set(ptr %load_thread_ptr, i32 %addtmp)
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
  br label %while_cond8

while_cond8:                                      ; preds = %while_body9, %entry5
  %load_counter12 = load ptr, ptr %counter6, align 8
  %call_thread_state_get13 = call i32 @wpp_thread_state_get(ptr %load_counter12)
  %lttmp14 = icmp slt i32 %call_thread_state_get13, 3
  br i1 %lttmp14, label %while_body9, label %while_end10

while_body9:                                      ; preds = %while_cond8
  %load_counter15 = load ptr, ptr %counter6, align 8
  %call_thread_state_get16 = call i32 @wpp_thread_state_get(ptr %load_counter15)
  %addtmp17 = add i32 %call_thread_state_get16, 1
  %load_thread_ptr18 = load ptr, ptr %counter6, align 8
  call void @wpp_thread_state_set(ptr %load_thread_ptr18, i32 %addtmp17)
  br label %while_cond8

while_end10:                                      ; preds = %while_cond8
  br label %after_loop11

after_loop11:                                     ; preds = %while_end10
  %load_counter19 = load ptr, ptr %counter6, align 8
  %call_thread_state_get20 = call i32 @wpp_thread_state_get(ptr %load_counter19)
  call void @wpp_print_i32(i32 %call_thread_state_get20)
  ret i32 0
}

define i32 @bgTask() {
entry:
  call void @wpp_print_value(ptr @strlit_0, i32 0)
  ret i32 0

entry1:                                           ; No predecessors!
  call void @wpp_print_value(ptr @strlit_1, i32 0)
  ret i32 0
}

declare void @wpp_print_i32(i32)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async
}

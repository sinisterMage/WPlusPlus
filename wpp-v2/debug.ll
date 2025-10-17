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
  %add = alloca ptr, align 8
  store ptr @0, ptr %add, align 8
  %load_fnptr_add = load ptr, ptr %add, align 8
  %call_indirect_add = call i32 %load_fnptr_add(i32 2, i32 3)
  call void @wpp_print_i32(i32 %call_indirect_add)
  call void @wpp_thread_join_all()
  ret i32 0
}

define i32 @0(i32 %0, i32 %1) {
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
}

declare void @wpp_return(i32)

declare void @wpp_print_i32(i32)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async
}

; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [34 x i8] c"Thread acquired lock and crashed!\00"
@strlit_1 = private constant [34 x i8] c"Thread acquired lock and crashed!\00"
@strlit_2 = private constant [13 x i8] c"Before spawn\00"
@strlit_3 = private constant [38 x i8] c"Main thread alive after worker crash!\00"

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
  call void @wpp_print_value(ptr @strlit_2, i32 0)
  %call_thread_spawn_gc = call ptr @wpp_thread_spawn_gc(ptr @worker)
  br i1 true, label %join_thread, label %cont_thread

join_thread:                                      ; preds = %entry
  call void @wpp_thread_join(ptr %call_thread_spawn_gc)
  br label %cont_thread

cont_thread:                                      ; preds = %join_thread, %entry
  call void @wpp_print_value(ptr @strlit_3, i32 0)
  call void @wpp_thread_join_all()
  ret i32 0
}

define i32 @worker() {
entry:
  %m = alloca ptr, align 8
  %call_mutex_new = call ptr @wpp_mutex_new(i32 0)
  store ptr %call_mutex_new, ptr %m, align 8
  %load_m = load ptr, ptr %m, align 8
  %call_thread_state_get = call ptr @wpp_thread_state_get(ptr %load_m)
  call void @wpp_mutex_lock(ptr %call_thread_state_get, i32 1)
  call void @wpp_print_value(ptr @strlit_0, i32 0)
  ret i32 0

entry1:                                           ; No predecessors!
  %m2 = alloca ptr, align 8
  %call_mutex_new3 = call ptr @wpp_mutex_new(i32 0)
  store ptr %call_mutex_new3, ptr %m2, align 8
  %load_m4 = load ptr, ptr %m2, align 8
  %call_thread_state_get5 = call ptr @wpp_thread_state_get(ptr %load_m4)
  call void @wpp_mutex_lock(ptr %call_thread_state_get5, i32 1)
  call void @wpp_print_value(ptr @strlit_1, i32 0)
  ret i32 0
}

declare void @wpp_print_i32(i32)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async
}

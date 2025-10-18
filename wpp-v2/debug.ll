; ModuleID = 'wpp_module'
source_filename = "wpp_module"

%Dog = type { i32 }

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [5 x i8] c"woof\00"

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
  %Dog_ptr = alloca %Dog, align 8
  %d = alloca ptr, align 8
  store ptr %Dog_ptr, ptr %d, align 8
  %d_load = load ptr, ptr %d, align 8
  %call_Dog.bark = call i32 @Dog.Dog.bark(ptr %d_load)
  call void @wpp_thread_join_all()
  ret i32 0
}

define i32 @Dog.Dog.new(ptr %0, i32 %1) {
entry:
  %me = alloca ptr, align 8
  store ptr %0, ptr %me, align 8
  %age = alloca i32, align 4
  store i32 %1, ptr %age, align 4
  %load_age = load i32, ptr %age, align 4
  store i32 %load_age, ptr %age, align 4
  ret i32 0
}

define i32 @Dog.Dog.bark(ptr %0) {
entry:
  %me = alloca ptr, align 8
  store ptr %0, ptr %me, align 8
  call void @wpp_print_value_basic(ptr @strlit_0, i32 6)
  %ret_tmp = alloca i32, align 4
  store i32 0, ptr %ret_tmp, align 4
  call void @wpp_return(ptr %ret_tmp, i32 1)
  ret i32 0

after_return:                                     ; No predecessors!
  ret i32 0
}

declare void @wpp_print_value_basic(ptr, i32)

declare void @wpp_return(ptr, i32)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async
}

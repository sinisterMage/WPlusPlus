; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [22 x i8] c"Handling HTTP request\00"
@strlit_1 = private constant [23 x i8] c"Handling HTTP response\00"
@strlit_2 = private constant [12 x i8] c"Hello doggo\00"
@strlit_3 = private constant [12 x i8] c"Hello kitty\00"

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

define i32 @handleRequest__Request(i32 %0) {
entry:
  %req = alloca i32, align 4
  store i32 %0, ptr %req, align 4
  call void @wpp_print_value_basic(ptr @strlit_0, i32 6)
  ret i32 0
}

declare void @wpp_print_value_basic(ptr, i32)

define i32 @handleResponse__Response(i32 %0) {
entry:
  %res = alloca i32, align 4
  store i32 %0, ptr %res, align 4
  call void @wpp_print_value_basic(ptr @strlit_1, i32 6)
  ret i32 0
}

define i32 @greet__Dog(i32 %0) {
entry:
  %animal = alloca i32, align 4
  store i32 %0, ptr %animal, align 4
  call void @wpp_print_value_basic(ptr @strlit_2, i32 6)
  ret i32 0
}

define i32 @greet__Cat(i32 %0) {
entry:
  %animal = alloca i32, align 4
  store i32 %0, ptr %animal, align 4
  call void @wpp_print_value_basic(ptr @strlit_3, i32 6)
  ret i32 0
}

declare i32 @__submodule_stub()

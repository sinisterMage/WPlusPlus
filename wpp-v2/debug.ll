; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [25 x i8] c"hello from W++ endpoint!\00"
@strlit_1 = private constant [25 x i8] c"hello from W++ endpoint!\00"
@strlit_2 = private constant [4 x i8] c"/hi\00"
@strlit_3 = private constant [20 x i8] c"https://example.com\00"

declare void @wpp_print_value(ptr, i32)

declare void @wpp_print_array(ptr)

declare void @wpp_print_object(ptr)

declare i32 @wpp_http_get(ptr)

declare void @wpp_register_endpoint(ptr, ptr)

declare void @wpp_start_server(i32)

define i32 @main_async() {
entry:
  %exc_flag = alloca i32, align 4
  store i32 0, ptr %exc_flag, align 4
  %exc_val_i32 = alloca i32, align 4
  store i32 0, ptr %exc_val_i32, align 4
  %exc_val_str = alloca ptr, align 8
  store ptr null, ptr %exc_val_str, align 8
  call void @wpp_register_endpoint(ptr @strlit_2, ptr @hello)
  call void @wpp_start_server(i32 8080)
  %res = alloca i32, align 4
  %call_http_get = call i32 @wpp_http_get(ptr @strlit_3)
  store i32 %call_http_get, ptr %res, align 4
  %load_res = load i32, ptr %res, align 4
  %int_as_ptr = inttoptr i32 %load_res to ptr
  call void @wpp_print_value(ptr %int_as_ptr, i32 0)
  ret i32 0
}

define i32 @hello() {
entry:
  call void @wpp_print_value(ptr @strlit_0, i32 0)
  ret i32 0

entry1:                                           ; No predecessors!
  call void @wpp_print_value(ptr @strlit_1, i32 0)
  ret i32 0
}

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async

after_return:                                     ; No predecessors!
  ret i32 0
}

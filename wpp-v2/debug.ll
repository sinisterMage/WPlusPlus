; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [25 x i8] c"https://httpbin.org/post\00"
@strlit_1 = private constant [19 x i8] c"{ \22msg\22: \22hello\22 }\00"
@strlit_2 = private constant [8 x i8] c"status:\00"
@strlit_3 = private constant [6 x i8] c"body:\00"

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
  %res = alloca i32, align 4
  %call_http.post = call i32 @wpp_http_post(ptr @strlit_0, ptr @strlit_1)
  store i32 %call_http.post, ptr %res, align 4
  call void @wpp_print_value(ptr @strlit_2, i32 0)
  %load_res = load i32, ptr %res, align 4
  %call_http_status = call i32 @wpp_http_status(i32 %load_res)
  call void @wpp_print_i32(i32 %call_http_status)
  call void @wpp_print_value(ptr @strlit_3, i32 0)
  %load_res1 = load i32, ptr %res, align 4
  %call_http_body = call ptr @wpp_http_body(i32 %load_res1)
  call void @wpp_print_value(ptr %call_http_body, i32 0)
  ret i32 0
}

declare i32 @wpp_http_post(ptr, ptr)

declare void @wpp_print_i32(i32)

declare i32 @wpp_http_status(i32)

declare ptr @wpp_http_body(i32)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async

after_return:                                     ; No predecessors!
  ret i32 0
}

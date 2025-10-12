; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [20 x i8] c"https://example.com\00"
@strlit_1 = private constant [25 x i8] c"https://httpbin.org/post\00"
@strlit_2 = private constant [25 x i8] c"{ 'msg': 'hello gamer' }\00"
@strlit_3 = private constant [24 x i8] c"https://httpbin.org/put\00"
@strlit_4 = private constant [19 x i8] c"{ 'update': true }\00"
@strlit_5 = private constant [26 x i8] c"https://httpbin.org/patch\00"
@strlit_6 = private constant [21 x i8] c"{ 'patching': true }\00"
@strlit_7 = private constant [27 x i8] c"https://httpbin.org/delete\00"
@strlit_8 = private constant [21 x i8] c"done making requests\00"

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
  %res1 = alloca i32, align 4
  %call_http_get = call i32 @wpp_http_get(ptr @strlit_0)
  store i32 %call_http_get, ptr %res1, align 4
  %res2 = alloca i32, align 4
  %call_http.post = call i32 @wpp_http_post(ptr @strlit_1, ptr @strlit_2)
  store i32 %call_http.post, ptr %res2, align 4
  %res3 = alloca i32, align 4
  %call_http.put = call i32 @wpp_http_put(ptr @strlit_3, ptr @strlit_4)
  store i32 %call_http.put, ptr %res3, align 4
  %res4 = alloca i32, align 4
  %call_http.patch = call i32 @wpp_http_patch(ptr @strlit_5, ptr @strlit_6)
  store i32 %call_http.patch, ptr %res4, align 4
  %res5 = alloca i32, align 4
  %call_http.delete = call i32 @wpp_http_delete(ptr @strlit_7)
  store i32 %call_http.delete, ptr %res5, align 4
  call void @wpp_print_value(ptr @strlit_8, i32 0)
  ret i32 0
}

declare i32 @wpp_http_post(ptr, ptr)

declare i32 @wpp_http_put(ptr, ptr)

declare i32 @wpp_http_patch(ptr, ptr)

declare i32 @wpp_http_delete(ptr)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async

after_return:                                     ; No predecessors!
  ret i32 0
}

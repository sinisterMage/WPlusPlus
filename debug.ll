; ModuleID = 'wpp_module'
source_filename = "wpp_module"

%Dog = type {}
%Cat = type {}

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [6 x i8] c"Woof!\00"
@strlit_1 = private constant [6 x i8] c"Meow!\00"
@strlit_2 = private constant [13 x i8] c"Hello doggo!\00"
@strlit_3 = private constant [13 x i8] c"Hello kitty!\00"
@strlit_4 = private constant [24 x i8] c"Processing HTTP request\00"
@strlit_5 = private constant [25 x i8] c"Processing HTTP response\00"
@strlit_6 = private constant [7 x i8] c"200 OK\00"
@strlit_7 = private constant [14 x i8] c"404 Not Found\00"
@strlit_8 = private constant [26 x i8] c"500 Internal Server Error\00"
@strlit_9 = private constant [14 x i8] c"2xx - Success\00"
@strlit_10 = private constant [19 x i8] c"4xx - Client Error\00"
@strlit_11 = private constant [19 x i8] c"5xx - Server Error\00"
@strlit_12 = private constant [44 x i8] c"=== W++ Enhanced Multiple Dispatch Demo ===\00"
@strlit_13 = private constant [1 x i8] zeroinitializer
@strlit_14 = private constant [29 x i8] c"--- Entity Type Dispatch ---\00"
@strlit_15 = private constant [1 x i8] zeroinitializer
@strlit_16 = private constant [29 x i8] c"--- Object Type Dispatch ---\00"
@strlit_17 = private constant [4 x i8] c"GET\00"
@objkey_0 = private constant [7 x i8] c"method\00"
@strlit_18 = private constant [11 x i8] c"/api/users\00"
@objkey_1 = private constant [4 x i8] c"url\00"
@objkey_0.1 = private constant [7 x i8] c"status\00"
@strlit_19 = private constant [8 x i8] c"Success\00"
@objkey_1.2 = private constant [5 x i8] c"body\00"
@strlit_20 = private constant [1 x i8] zeroinitializer
@strlit_21 = private constant [42 x i8] c"--- HTTP Status Code Literal Dispatch ---\00"
@strlit_22 = private constant [1 x i8] zeroinitializer
@strlit_23 = private constant [40 x i8] c"--- HTTP Status Code Range Dispatch ---\00"
@strlit_24 = private constant [1 x i8] zeroinitializer
@strlit_25 = private constant [22 x i8] c"=== Demo Complete ===\00"

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

define i32 @Dog.Dog.speak(ptr %0) {
entry:
  %me = alloca ptr, align 8
  store ptr %0, ptr %me, align 8
  call void @wpp_print_value_basic(ptr @strlit_0, i32 6)
  ret i32 0
}

declare void @wpp_print_value_basic(ptr, i32)

define i32 @Cat.Cat.speak(ptr %0) {
entry:
  %me = alloca ptr, align 8
  store ptr %0, ptr %me, align 8
  call void @wpp_print_value_basic(ptr @strlit_1, i32 6)
  ret i32 0
}

define i32 @greet__entity_Dog(ptr %0) {
entry:
  %animal = alloca ptr, align 8
  store ptr %0, ptr %animal, align 8
  call void @wpp_print_value_basic(ptr @strlit_2, i32 6)
  ret i32 0
}

define i32 @greet__entity_Cat(ptr %0) {
entry:
  %animal = alloca ptr, align 8
  store ptr %0, ptr %animal, align 8
  call void @wpp_print_value_basic(ptr @strlit_3, i32 6)
  ret i32 0
}

define i32 @process__obj_Request(ptr %0) {
entry:
  %req = alloca ptr, align 8
  store ptr %0, ptr %req, align 8
  call void @wpp_print_value_basic(ptr @strlit_4, i32 6)
  ret i32 0
}

define i32 @process__obj_Response(ptr %0) {
entry:
  %res = alloca ptr, align 8
  store ptr %0, ptr %res, align 8
  call void @wpp_print_value_basic(ptr @strlit_5, i32 6)
  ret i32 0
}

define i32 @handleCode__http_200(i32 %0) {
entry:
  %status = alloca i32, align 4
  store i32 %0, ptr %status, align 4
  call void @wpp_print_value_basic(ptr @strlit_6, i32 6)
  ret i32 0
}

define i32 @handleCode__http_404(i32 %0) {
entry:
  %status = alloca i32, align 4
  store i32 %0, ptr %status, align 4
  call void @wpp_print_value_basic(ptr @strlit_7, i32 6)
  ret i32 0
}

define i32 @handleCode__http_500(i32 %0) {
entry:
  %status = alloca i32, align 4
  store i32 %0, ptr %status, align 4
  call void @wpp_print_value_basic(ptr @strlit_8, i32 6)
  ret i32 0
}

define i32 @handleRange__http_2xx(i32 %0) {
entry:
  %code = alloca i32, align 4
  store i32 %0, ptr %code, align 4
  call void @wpp_print_value_basic(ptr @strlit_9, i32 6)
  ret i32 0
}

define i32 @handleRange__http_4xx(i32 %0) {
entry:
  %code = alloca i32, align 4
  store i32 %0, ptr %code, align 4
  call void @wpp_print_value_basic(ptr @strlit_10, i32 6)
  ret i32 0
}

define i32 @handleRange__http_5xx(i32 %0) {
entry:
  %code = alloca i32, align 4
  store i32 %0, ptr %code, align 4
  call void @wpp_print_value_basic(ptr @strlit_11, i32 6)
  ret i32 0
}

define i32 @main() {
entry:
  call void @wpp_print_value_basic(ptr @strlit_12, i32 6)
  call void @wpp_print_value_basic(ptr @strlit_13, i32 6)
  call void @wpp_print_value_basic(ptr @strlit_14, i32 6)
  %Dog_ptr = alloca %Dog, align 8
  %dog = alloca ptr, align 8
  store ptr %Dog_ptr, ptr %dog, align 8
  %Cat_ptr = alloca %Cat, align 8
  %cat = alloca ptr, align 8
  store ptr %Cat_ptr, ptr %cat, align 8
  %load_dog = load ptr, ptr %dog, align 8
  %call_greet = call i32 @greet__entity_Dog(ptr %load_dog)
  %load_cat = load ptr, ptr %cat, align 8
  %call_greet1 = call i32 @greet__entity_Cat(ptr %load_cat)
  call void @wpp_print_value_basic(ptr @strlit_15, i32 6)
  call void @wpp_print_value_basic(ptr @strlit_16, i32 6)
  %request = alloca ptr, align 8
  %obj_malloc = call ptr @malloc(i64 ptrtoint (ptr getelementptr ({ i32, ptr, ptr }, ptr null, i32 1) to i64))
  %keys_malloc = call ptr @malloc(i64 mul (i64 ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64), i64 2))
  %vals_malloc = call ptr @malloc(i64 mul (i64 ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64), i64 2))
  %key_slot = getelementptr ptr, ptr %keys_malloc, i32 0
  store ptr @objkey_0, ptr %key_slot, align 8
  %val_slot = getelementptr i32, ptr %vals_malloc, i32 0
  store i32 0, ptr %val_slot, align 4
  %key_slot2 = getelementptr ptr, ptr %keys_malloc, i32 1
  store ptr @objkey_1, ptr %key_slot2, align 8
  %val_slot3 = getelementptr i32, ptr %vals_malloc, i32 1
  store i32 0, ptr %val_slot3, align 4
  %f0 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc, i32 0, i32 0
  store i32 2, ptr %f0, align 4
  %f1 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc, i32 0, i32 1
  store ptr %keys_malloc, ptr %f1, align 8
  %f2 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc, i32 0, i32 2
  store ptr %vals_malloc, ptr %f2, align 8
  store ptr %obj_malloc, ptr %request, align 8
  %response = alloca ptr, align 8
  %obj_malloc4 = call ptr @malloc(i64 ptrtoint (ptr getelementptr ({ i32, ptr, ptr }, ptr null, i32 1) to i64))
  %keys_malloc5 = call ptr @malloc(i64 mul (i64 ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64), i64 2))
  %vals_malloc6 = call ptr @malloc(i64 mul (i64 ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64), i64 2))
  %key_slot7 = getelementptr ptr, ptr %keys_malloc5, i32 0
  store ptr @objkey_0.1, ptr %key_slot7, align 8
  %val_slot8 = getelementptr i32, ptr %vals_malloc6, i32 0
  store i32 200, ptr %val_slot8, align 4
  %key_slot9 = getelementptr ptr, ptr %keys_malloc5, i32 1
  store ptr @objkey_1.2, ptr %key_slot9, align 8
  %val_slot10 = getelementptr i32, ptr %vals_malloc6, i32 1
  store i32 0, ptr %val_slot10, align 4
  %f011 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc4, i32 0, i32 0
  store i32 2, ptr %f011, align 4
  %f112 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc4, i32 0, i32 1
  store ptr %keys_malloc5, ptr %f112, align 8
  %f213 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc4, i32 0, i32 2
  store ptr %vals_malloc6, ptr %f213, align 8
  store ptr %obj_malloc4, ptr %response, align 8
  %load_request = load ptr, ptr %request, align 8
  %call_process = call i32 @process__obj_Request(ptr %load_request)
  %load_response = load ptr, ptr %response, align 8
  %call_process14 = call i32 @process__obj_Response(ptr %load_response)
  call void @wpp_print_value_basic(ptr @strlit_20, i32 6)
  call void @wpp_print_value_basic(ptr @strlit_21, i32 6)
  %call_handleCode = call i32 @handleCode__http_200(i32 200)
  %call_handleCode15 = call i32 @handleCode__http_404(i32 404)
  %call_handleCode16 = call i32 @handleCode__http_500(i32 500)
  call void @wpp_print_value_basic(ptr @strlit_22, i32 6)
  call void @wpp_print_value_basic(ptr @strlit_23, i32 6)
  %call_handleRange = call i32 @handleRange__http_2xx(i32 201)
  %call_handleRange17 = call i32 @handleRange__http_4xx(i32 403)
  %call_handleRange18 = call i32 @handleRange__http_5xx(i32 503)
  call void @wpp_print_value_basic(ptr @strlit_24, i32 6)
  call void @wpp_print_value_basic(ptr @strlit_25, i32 6)
  ret i32 0
}

declare ptr @malloc(i64)

declare i32 @__submodule_stub()

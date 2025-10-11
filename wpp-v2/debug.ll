; ModuleID = 'wpp_module'
source_filename = "wpp_module"

@_exception_flag = global i1 false
@_exception_value_i32 = global i32 0
@_exception_value_str = global ptr null
@objkey_0 = private constant [2 x i8] c"x\00"
@objkey_1 = private constant [2 x i8] c"y\00"
@fmt_d = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@fmt_d.1 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

define i32 @main_async() {
entry:
  %exc_flag = alloca i32, align 4
  store i32 0, ptr %exc_flag, align 4
  %exc_val_i32 = alloca i32, align 4
  store i32 0, ptr %exc_val_i32, align 4
  %exc_val_str = alloca ptr, align 8
  store ptr null, ptr %exc_val_str, align 8
  %arr = alloca i32, align 4
  %arr_malloc = call ptr @malloc(i64 mul (i64 ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64), i64 5))
  store i32 4, ptr %arr_malloc, align 4
  %elem_ptr = getelementptr i32, ptr %arr_malloc, i32 1
  store i32 1, ptr %elem_ptr, align 4
  %elem_ptr1 = getelementptr i32, ptr %arr_malloc, i32 2
  store i32 2, ptr %elem_ptr1, align 4
  %elem_ptr2 = getelementptr i32, ptr %arr_malloc, i32 3
  store i32 3, ptr %elem_ptr2, align 4
  %elem_ptr3 = getelementptr i32, ptr %arr_malloc, i32 4
  store i32 4, ptr %elem_ptr3, align 4
  store ptr %arr_malloc, ptr %arr, align 8
  %obj = alloca i32, align 4
  %obj_malloc = call ptr @malloc(i64 ptrtoint (ptr getelementptr ({ i32, ptr, ptr }, ptr null, i32 1) to i64))
  %keys_malloc = call ptr @malloc(i64 mul (i64 ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64), i64 2))
  %vals_malloc = call ptr @malloc(i64 mul (i64 ptrtoint (ptr getelementptr (i32, ptr null, i32 1) to i64), i64 2))
  %key_slot = getelementptr ptr, ptr %keys_malloc, i32 0
  store ptr @objkey_0, ptr %key_slot, align 8
  %val_slot = getelementptr i32, ptr %vals_malloc, i32 0
  store i32 10, ptr %val_slot, align 4
  %key_slot4 = getelementptr ptr, ptr %keys_malloc, i32 1
  store ptr @objkey_1, ptr %key_slot4, align 8
  %val_slot5 = getelementptr i32, ptr %vals_malloc, i32 1
  store i32 20, ptr %val_slot5, align 4
  %f0 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc, i32 0, i32 0
  store i32 2, ptr %f0, align 4
  %f1 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc, i32 0, i32 1
  store ptr %keys_malloc, ptr %f1, align 8
  %f2 = getelementptr inbounds { i32, ptr, ptr }, ptr %obj_malloc, i32 0, i32 2
  store ptr %vals_malloc, ptr %f2, align 8
  store ptr %obj_malloc, ptr %obj, align 8
  %load_arr = load i32, ptr %arr, align 4
  %call_printf_int = call i32 (ptr, ...) @printf(ptr @fmt_d, i32 %load_arr)
  %load_obj = load i32, ptr %obj, align 4
  %call_printf_int6 = call i32 (ptr, ...) @printf(ptr @fmt_d.1, i32 %load_obj)
  ret i32 0
}

declare ptr @malloc(i64)

declare i32 @printf(ptr, ...)

declare i32 @puts(ptr)

define i32 @main() {
entry:
  %call_main_async = call i32 @main_async()
  ret i32 %call_main_async

after_return:                                     ; No predecessors!
  ret i32 0
}

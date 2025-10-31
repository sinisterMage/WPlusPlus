; ModuleID = 'wpp_module'
source_filename = "wpp_module"

%Dog = type {}
%Cat = type {}

@_wpp_exc_flag = global i1 false
@_wpp_exc_i32 = global i32 0
@_wpp_exc_str = global ptr null
@strlit_0 = private constant [6 x i8] c"Woof!\00"
@strlit_1 = private constant [6 x i8] c"Meow!\00"
@strlit_2 = private constant [12 x i8] c"Hello doggo\00"
@strlit_3 = private constant [12 x i8] c"Hello kitty\00"
@strlit_4 = private constant [24 x i8] c"Testing entity dispatch\00"

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

define i32 @main() {
entry:
  call void @wpp_print_value_basic(ptr @strlit_4, i32 6)
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
  ret i32 %call_greet1
}

declare i32 @__submodule_stub()

;; rt.wat start
;; ============

;; runtime assumes the following variables are available and pre-initialized:
;; (global $stack_bottom (mut i32) (i32.const {}))
;; (global $stack_top i32 (i32.const {}))
;; (global $heap_start (mut i32) (i32.const {}))

(type $user_fun (func (param i32) (result i32)))

;; indirect function call via function object, possibly switching to a new
;; stack for the call
(func $call_function (param $func_addr i32) (param $params_addr i32) (result i32) (local $retval i32)
    ;; if we end up here, there is no custom stack for the call
    local.get $params_addr
    ;; load function index
    local.get $func_addr
    i32.const 4
    i32.add
    i32.load
    call_indirect (type $user_fun)
)

(func $call_function_old (param $func_addr i32) (param $params_addr i32) (result i32) (local $retval i32)
    (block $without_stack
        (block $with_stack
            ;; load stack address
            local.get $func_addr
            i32.const 8
            i32.add
            i32.load
            br_if $with_stack
            ;; if we end up here, there is no custom stack for the call
            local.get $params_addr
            ;; load function index
            local.get $func_addr
            i32.const 4
            i32.add
            i32.load
            call_indirect
			local.set $retval
            br $without_stack
            )
        ;; if we end up here there is a custom stack
        ;; enter stack
        local.get $func_addr
        i32.const 8
        i32.add
        i32.load
        call $push_stack

        local.get $params_addr
        ;; load function index
        local.get $func_addr
        i32.const 4
        i32.add
        i32.load
        call_indirect
		local.set $retval

        ;; exit stack
        call $pop_stack
        )
	local.get $retval
)

(func $inc_stack_bottom (param $by i32)
    global.get $stack_bottom
    local.get $by
    i32.add
    global.set $stack_bottom)

;; allocates space for a new stack, and returns the start
(func $alloc_new_stack (result i32)
    global.get $heap_start ;; return value
    global.get $heap_start ;; new value to be written
    i32.const 1024 ;; allocate 1024 bytes of stack
    i32.add
    global.set $heap_start
)

;; enter a stack and remember the previous stack on the new stack
(func $push_stack (param $new_stack i32)
    ;; back up previous values on the new stack
    local.get $new_stack
    global.get $stack_bottom
    i32.store

    ;; then use the area after the new values as the new stack
    local.get $new_stack
    i32.const 4
    i32.add
    global.set $stack_bottom
)

;; pop a return stack from below the stack bottom and switch to it
(func $pop_stack
    ;; get previous stack bottom and local offset and restore them
    global.get $stack_bottom
    i32.const 4
    i32.sub
    i32.load
    global.set $stack_bottom
)

(func $make_function (param $table_idx i32) (param $stack i32) (result i32)
    global.get $heap_start ;; return value
    global.get $heap_start ;; target of later store for type
    global.get $heap_start ;; target of later store for table index
    i32.const 4
    i32.add
    global.get $heap_start ;; target of later store for stack
    i32.const 8
    i32.add
    global.get $heap_start ;; calculate and set new top
    i32.const 12 ;; actual length includes space for type tag, table index and stack
    i32.add
    global.set $heap_start
    local.get $stack
    i32.store
    local.get $table_idx
    i32.store
    i32.const 32 ;; 32 is type for function (=0b10_0000)
    i32.store
)

;; allocates a sized thing with the given length and type tag and returns the beginning of the allocation, where type and length are already set and the rest is uninitialized
(func $alloc_sized (param $len_without_type_and_len i32) (param $type i32) (result i32)
    global.get $heap_start ;; return value
    global.get $heap_start ;; target of later store for type
    global.get $heap_start ;; target of later store for length
    i32.const 4
    i32.add
    global.get $heap_start ;; calculate and set new top
    local.get $len_without_type_and_len
    i32.const 8 ;; actual length includes space for type tag and length
    i32.add
    i32.add
    global.set $heap_start
    local.get $len_without_type_and_len
    i32.store
    local.get $type
    i32.store
)

(func $concat_strings (param $left_string_address i32) (param $right_string_address i32) (result i32) (local $result_addr i32)
    local.get $left_string_address
    i32.const 4
    i32.add
    i32.load ;; load left length
    local.get $right_string_address
    i32.const 4
    i32.add
    i32.load ;; load right length
    i32.add ;; combined length
    i32.const 8 ;; type for character data (=0b1000)
    call $alloc_sized
    local.set $result_addr

    ;; dst address is start of character data of the allocation
    local.get $result_addr
    i32.const 8
    i32.add

    ;; source is character data of left string
    local.get $left_string_address
    i32.const 8
    i32.add

    ;; count is left character count
    local.get $left_string_address
    i32.const 4
    i32.add
    i32.load
    memory.copy

    ;; dst address is allocation data plus type tag and size plus left size
    local.get $result_addr
    i32.const 8
    i32.add
    local.get $left_string_address
    i32.const 4
    i32.add
    i32.load
    i32.add

    ;; source is right string character data
    local.get $right_string_address
    i32.const 8
    i32.add

    ;; count is taken from the right
    local.get $right_string_address
    i32.const 4
    i32.add
    i32.load
    memory.copy

    local.get $result_addr ;; return value is start of the final allocation
)

(func $make_num (param $value i32) (result i32)
    global.get $heap_start ;; return value
    global.get $heap_start ;; target of later store for type
    global.get $heap_start ;; target of later store for value
    i32.const 4
    i32.add
    global.get $heap_start ;; calculate and set new top
    i32.const 8 ;; actual length includes space for type tag and actual number
    i32.add
    global.set $heap_start
    local.get $value
    i32.store
    i32.const 4 ;; 4 is type for number (=0b100)
    i32.store
)

;; rt.wat end
;; ==========

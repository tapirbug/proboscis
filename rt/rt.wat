;; rt.wat start
;; ============

;; runtime assumes the following variables are available and pre-initialized:
;; (global $stack_bottom (mut i32) (i32.const {}))
;; (global $stack_top i32 (i32.const {}))
;; (global $heap_start (mut i32) (i32.const {}))

(type $user_fun (func (param i32) (param i32) (result i32)))

;; indirect function call via function object
(func $call_function (param $func_addr i32) (param $params_addr i32) (result i32) (local $retval i32)
    ;; load params as first parameter
    local.get $params_addr
    ;; load persistent base address as second parameter
    local.get $func_addr
    i32.const 8
    i32.add
    i32.load
    ;; load function index
    local.get $func_addr
    i32.const 4
    i32.add
    i32.load
    call_indirect (type $user_fun)
)

(func $inc_stack_bottom (param $by i32)
    global.get $stack_bottom
    local.get $by
    i32.add
    global.set $stack_bottom)

;; allocates space on the heap and returns the start
(func $alloc_heap (param $bytes i32) (result i32)
    global.get $heap_start ;; return value
    global.get $heap_start ;; new value to be written
    local.get $bytes
    i32.add
    global.set $heap_start
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

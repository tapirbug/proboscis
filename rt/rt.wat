;; rt.wat start
;; ============

;; runtime assumes the following variables are available and pre-initialized:
;; (global $stack_bottom i32 (i32.const {}))
;; (global $stack_top (mut i32) (i32.const {}))
;; (global $local_offset (mut i32) (i32.const {}))
;; (global $heap_start (mut i32) (i32.const {}))

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
    i32.const 2 ;; type for character data
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
    i32.const 3 ;; 3 is type for number
    i32.store
)

;; rt.wat end
;; ==========

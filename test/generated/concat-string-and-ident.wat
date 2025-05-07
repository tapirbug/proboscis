(module
	(import "js" "mem" (memory 1))
	(import "console" "log" (func $log (param i32 i32)))
	(data (i32.const 0) "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\04\00\00\00\01\00\00\00T\10\00\00\00\04\00\00\00\07\00\00\00Tapirus\02\00\00\00\0B\00\00\00 terrestris")
	(global $stack_bottom (mut i32) (i32.const 63))
	(global $stack_top i32 (i32.const 10303))
	(global $local_offset (mut i32) (i32.const 63))
	(global $heap_start (mut i32) (i32.const 10303))

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

	(func $fun0 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 4
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; CallPrint { string: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		local.set 1
		local.get 1
		i32.const 8
		i32.add
		local.get 1
		i32.const 4
		i32.add
		i32.load
		call $log
		;; Return { value: PlaceAddress { mode: Global, offset: 12 } }
		i32.const 12
		i32.load
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
	)
	(func $fun1 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 4
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; LoadTypeTag { of: PlaceAddress { mode: Local, offset: 0 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.load
		call $make_num
		i32.store
		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
	)
	(func $fun2 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 8
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 4 } }
		global.get $local_offset
		i32.const 4
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; ConcatStringLike { left: PlaceAddress { mode: Local, offset: 0 }, right: PlaceAddress { mode: Local, offset: 4 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $local_offset
		i32.const 4
		i32.add
		i32.load
		call $concat_strings
		i32.store
		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 8
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 8
		i32.sub
		global.set $stack_bottom
	)
	(func $fun3 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 8
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 4 } }
		global.get $local_offset
		i32.const 4
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; Cons { car: PlaceAddress { mode: Local, offset: 0 }, cdr: PlaceAddress { mode: Local, offset: 4 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $heap_start
		global.get $heap_start
		i32.const 12
		i32.add
		global.set $heap_start
		local.set 1
		local.get 1
		i32.const 1
		i32.store
		local.get 1
		i32.const 4
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.store
		local.get 1
		i32.const 8
		i32.add
		global.get $local_offset
		i32.const 4
		i32.add
		i32.load
		i32.store
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 1
		i32.store
		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 8
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 8
		i32.sub
		global.set $stack_bottom
	)
	(func $fun4 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 4
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; LoadCar { list: PlaceAddress { mode: Local, offset: 0 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.const 4
		i32.load
		i32.store
		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
	)
	(func $fun5 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 4
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; LoadCdr { list: PlaceAddress { mode: Local, offset: 0 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.const 8
		i32.load
		i32.store
		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
	)
	(func $fun6 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 8
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 4 } }
		global.get $local_offset
		i32.const 4
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; Add { left: PlaceAddress { mode: Local, offset: 0 }, right: PlaceAddress { mode: Local, offset: 4 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.const 4
		i32.add
		i32.load
		global.get $local_offset
		i32.const 4
		i32.add
		i32.load
		i32.const 4
		i32.add
		i32.load
		i32.add		call $make_num		i32.store		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 8
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 8
		i32.sub
		global.set $stack_bottom
	)
	(func $fun7 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 8
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 4 } }
		global.get $local_offset
		i32.const 4
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; Sub { left: PlaceAddress { mode: Local, offset: 0 }, right: PlaceAddress { mode: Local, offset: 4 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.const 4
		i32.add
		i32.load
		global.get $local_offset
		i32.const 4
		i32.add
		i32.load
		i32.const 4
		i32.add
		i32.load
		i32.sub
		call $make_num
		i32.store
		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 8
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 8
		i32.sub
		global.set $stack_bottom
	)
	(func $fun8 (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 4
		i32.add
		global.set $stack_bottom
		;; ConsumeParam { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 0
		i32.const 8
		i32.add
		i32.load
		local.set 0
		;; NilIfZero { check: PlaceAddress { mode: Local, offset: 0 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.const 0
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.const 4
		i32.add
		i32.load
		select
		i32.store
		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
	)
	(func $fun9 (export "list") (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 4
		i32.add
		global.set $stack_bottom
		;; ConsumeRest { to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		local.get 0
		i32.store
		;; Return { value: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 4
		i32.sub
		global.set $stack_bottom
	)
	(func $fun10 (export "main") (param i32) (result i32) (local i32)
		global.get $stack_bottom
		i32.const 24
		i32.add
		global.set $stack_bottom
		;; LoadData { data: DataAddress(29), to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.const 29
		i32.store
		;; LoadData { data: DataAddress(44), to: PlaceAddress { mode: Local, offset: 4 } }
		global.get $local_offset
		i32.const 4
		i32.add
		i32.const 44
		i32.store
		;; WritePlace { from: PlaceAddress { mode: Global, offset: 12 }, to: PlaceAddress { mode: Local, offset: 8 } }
		global.get $local_offset
		i32.const 8
		i32.add
		i32.const 12
		i32.load
		i32.store
		;; Cons { car: PlaceAddress { mode: Local, offset: 4 }, cdr: PlaceAddress { mode: Local, offset: 8 }, to: PlaceAddress { mode: Local, offset: 8 } }
		global.get $heap_start
		global.get $heap_start
		i32.const 12
		i32.add
		global.set $heap_start
		local.set 1
		local.get 1
		i32.const 1
		i32.store
		local.get 1
		i32.const 4
		i32.add
		global.get $local_offset
		i32.const 4
		i32.add
		i32.load
		i32.store
		local.get 1
		i32.const 8
		i32.add
		global.get $local_offset
		i32.const 8
		i32.add
		i32.load
		i32.store
		global.get $local_offset
		i32.const 8
		i32.add
		local.get 1
		i32.store
		;; Cons { car: PlaceAddress { mode: Local, offset: 0 }, cdr: PlaceAddress { mode: Local, offset: 8 }, to: PlaceAddress { mode: Local, offset: 8 } }
		global.get $heap_start
		global.get $heap_start
		i32.const 12
		i32.add
		global.set $heap_start
		local.set 1
		local.get 1
		i32.const 1
		i32.store
		local.get 1
		i32.const 4
		i32.add
		global.get $local_offset
		i32.const 0
		i32.add
		i32.load
		i32.store
		local.get 1
		i32.const 8
		i32.add
		global.get $local_offset
		i32.const 8
		i32.add
		i32.load
		i32.store
		global.get $local_offset
		i32.const 8
		i32.add
		local.get 1
		i32.store
		;; Call { function: StaticFunctionAddress(2), params: PlaceAddress { mode: Local, offset: 8 }, to: PlaceAddress { mode: Local, offset: 12 } }
		global.get $local_offset
		global.get $local_offset
		i32.const 12
		i32.add
		global.get $local_offset
		i32.const 8
		i32.add
		i32.load
		global.get $stack_bottom
		global.set $local_offset
		call $fun2
		i32.store
		global.set $local_offset
		;; WritePlace { from: PlaceAddress { mode: Global, offset: 12 }, to: PlaceAddress { mode: Local, offset: 16 } }
		global.get $local_offset
		i32.const 16
		i32.add
		i32.const 12
		i32.load
		i32.store
		;; Cons { car: PlaceAddress { mode: Local, offset: 12 }, cdr: PlaceAddress { mode: Local, offset: 16 }, to: PlaceAddress { mode: Local, offset: 16 } }
		global.get $heap_start
		global.get $heap_start
		i32.const 12
		i32.add
		global.set $heap_start
		local.set 1
		local.get 1
		i32.const 1
		i32.store
		local.get 1
		i32.const 4
		i32.add
		global.get $local_offset
		i32.const 12
		i32.add
		i32.load
		i32.store
		local.get 1
		i32.const 8
		i32.add
		global.get $local_offset
		i32.const 16
		i32.add
		i32.load
		i32.store
		global.get $local_offset
		i32.const 16
		i32.add
		local.get 1
		i32.store
		;; Cons { car: PlaceAddress { mode: Global, offset: 25 }, cdr: PlaceAddress { mode: Local, offset: 16 }, to: PlaceAddress { mode: Local, offset: 16 } }
		global.get $heap_start
		global.get $heap_start
		i32.const 12
		i32.add
		global.set $heap_start
		local.set 1
		local.get 1
		i32.const 1
		i32.store
		local.get 1
		i32.const 4
		i32.add
		i32.const 25
		i32.load
		i32.store
		local.get 1
		i32.const 8
		i32.add
		global.get $local_offset
		i32.const 16
		i32.add
		i32.load
		i32.store
		global.get $local_offset
		i32.const 16
		i32.add
		local.get 1
		i32.store
		;; Call { function: StaticFunctionAddress(0), params: PlaceAddress { mode: Local, offset: 16 }, to: PlaceAddress { mode: Local, offset: 20 } }
		global.get $local_offset
		global.get $local_offset
		i32.const 20
		i32.add
		global.get $local_offset
		i32.const 16
		i32.add
		i32.load
		global.get $stack_bottom
		global.set $local_offset
		call $fun0
		i32.store
		global.set $local_offset
		;; Return { value: PlaceAddress { mode: Local, offset: 20 } }
		global.get $local_offset
		i32.const 20
		i32.add
		i32.load
		global.get $stack_bottom
		i32.const 24
		i32.sub
		global.set $stack_bottom
		return
		global.get $stack_bottom
		i32.const 24
		i32.sub
		global.set $stack_bottom
	)
)

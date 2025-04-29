(module
    (import "js" "mem" (memory 1))
    (import "console" "log" (func $log (param i32 i32)))
    (global $alloc_from (mut i32) (i32.const 32))
    (data (i32.const 0) "Hey the max is: ")
    (data (i32.const 16) "0123456789ABCDEF")

    (func $alloc (param $len i32) (result i32)
        global.get $alloc_from
        global.get $alloc_from
        local.get $len
        i32.add
        global.set $alloc_from)

    (func $concat (param $l_start i32) (param $l_len i32) (param $r_start i32) (param $r_len i32) (result i32 i32) (local $dst_start i32) (local $dst_len i32)
        local.get $l_len
        local.get $r_len
        i32.add
        local.set $dst_len
        local.get $dst_len
        call $alloc
        local.set $dst_start
        local.get $dst_start ;; destination
        local.get $l_start ;; source
        local.get $l_len ;; bytes to copy
        memory.copy
        local.get $dst_start
        local.get $l_len
        i32.add ;; second destination address is dst + left len
        local.get $r_start
        local.get $r_len
        memory.copy
        local.get $dst_start
        local.get $dst_len)

    (func $int2str (param $int i32) (result i32 i32) (local $str_write i32)
        i32.const 8
        call $alloc
        local.set $str_write
        local.get $str_write ;; destination address
        local.get $int
        i32.const 28
        i32.shr_u
        i32.const 15
        i32.and ;; 0-15 of highest digit
        i32.const 16 ;; offset of first static digit
        i32.add ;; 16 + (a >> 28) & 0xf is the source address
        i32.const 1 ;; length 1
        memory.copy

        local.get $str_write
        i32.const 1
        i32.add ;; destination address char 2
        local.get $int
        i32.const 24
        i32.shr_u
        i32.const 15
        i32.and ;; 0-15 of highest digit
        i32.const 16 ;; offset of first static digit
        i32.add ;; 16 + (a >> 24) & 0xf is the source address
        i32.const 1 ;; length 1
        memory.copy

        local.get $str_write
        i32.const 2
        i32.add ;; destination address char 3
        local.get $int
        i32.const 20
        i32.shr_u
        i32.const 15
        i32.and ;; 0-15 of highest digit
        i32.const 16 ;; offset of first static digit
        i32.add
        i32.const 1 ;; length 1
        memory.copy

        local.get $str_write
        i32.const 3
        i32.add ;; destination address char 4
        local.get $int
        i32.const 16
        i32.shr_u
        i32.const 15
        i32.and ;; 0-15 of highest digit
        i32.const 16 ;; offset of first static digit
        i32.add
        i32.const 1 ;; length 1
        memory.copy

        local.get $str_write
        i32.const 4
        i32.add ;; destination address char 5
        local.get $int
        i32.const 12
        i32.shr_u
        i32.const 15
        i32.and ;; 0-15 of highest digit
        i32.const 16 ;; offset of first static digit
        i32.add
        i32.const 1 ;; length 1
        memory.copy

        local.get $str_write
        i32.const 5
        i32.add ;; destination address char 6
        local.get $int
        i32.const 8
        i32.shr_u
        i32.const 15
        i32.and ;; 0-15 of highest digit
        i32.const 16 ;; offset of first static digit
        i32.add
        i32.const 1 ;; length 1
        memory.copy

        local.get $str_write
        i32.const 6
        i32.add ;; destination address char 7
        local.get $int
        i32.const 4
        i32.shr_u
        i32.const 15
        i32.and ;; 0-15 of highest digit
        i32.const 16 ;; offset of first static digit
        i32.add
        i32.const 1 ;; length 1
        memory.copy

        local.get $str_write
        i32.const 7
        i32.add ;; destination address char 8
        local.get $int
        i32.const 0
        i32.shr_u
        i32.const 15
        i32.and ;; 0-15 of highest digit
        i32.const 16 ;; offset of first static digit
        i32.add
        i32.const 1 ;; length 1
        memory.copy

        local.get $str_write
        i32.const 8)

    (func $max2
        (param i32) (param i32) (result i32)
        local.get 0
        local.get 1
        i32.gt_s
        (if (result i32)
            (then local.get 0)
            (else local.get 1)
        )
    )
    (func $main
        i32.const 0
        i32.const 16 ;; static string first
        i32.const 3
        i32.const 8
        call $max2
        call $int2str ;; then convert max to string
        call $concat ;; concat the two
        call $log ;; log the result
    )
    (export "main" (func $main)))
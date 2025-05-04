(module
	(import "js" "mem" (memory 1))
	(import "console" "log" (func $log (param i32 i32)))
	(data (i32.const 0) "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\01\00\00\00\01\00\00\00T\10\00\00\00\01\00\00\00\0F\00\00\00Tapirus indicus")
	(global $stack_bottom i32 (i32.const 52))
	(global $stack_top (mut i32) (i32.const 10292))
	(global $local_offset (mut i32) (i32.const 52))
	(global $heap_start (mut i32) (i32.const 10292))
	(func $fun0 (param i32) (result i32) (local i32)
		;; AllocPlaces { count: 1 }
		global.get $stack_top
		i32.const 4
		i32.add
		global.set $stack_top
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
		;; DeallocPlaces { count: 1 }
		global.get $stack_top
		i32.const 4
		i32.sub
		global.set $stack_top
		;; Return { value: PlaceAddress { mode: Global, offset: 12 } }
		i32.const 12
		i32.load
		return
	)
	(func $fun1 (export "main") (param i32) (result i32) (local i32)
		;; AllocPlaces { count: 1 }
		global.get $stack_top
		i32.const 4
		i32.add
		global.set $stack_top
		;; WritePlace { from: PlaceAddress { mode: Global, offset: 25 }, to: PlaceAddress { mode: Local, offset: 0 } }
		global.get $local_offset
		i32.const 0
		i32.add
		i32.const 25
		i32.load
		i32.store
		;; AllocPlaces { count: 1 }
		global.get $stack_top
		i32.const 4
		i32.add
		global.set $stack_top
		;; LoadData { data: DataAddress(29), to: PlaceAddress { mode: Local, offset: 4 } }
		global.get $local_offset
		i32.const 4
		i32.add
		i32.const 29
		i32.store
		;; AllocPlaces { count: 2 }
		global.get $stack_top
		i32.const 8
		i32.add
		global.set $stack_top
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
		i32.const 0
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
		i32.const 0
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
		;; Call { function: StaticFunctionAddress(0), params: PlaceAddress { mode: Local, offset: 8 }, to: PlaceAddress { mode: Local, offset: 12 } }
		global.get $local_offset
		global.get $local_offset
		i32.const 12
		i32.add
		global.get $local_offset
		i32.const 8
		i32.add
		i32.load
		global.get $stack_top
		global.set $local_offset
		call $fun0
		i32.store
		global.set $local_offset
		;; DeallocPlaces { count: 16 }
		global.get $stack_top
		i32.const 64
		i32.sub
		global.set $stack_top
		;; Return { value: PlaceAddress { mode: Global, offset: 12 } }
		i32.const 12
		i32.load
		return
	)
)

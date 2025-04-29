WAT := $(wildcard examples/webassembly/*.wat)
WASM := $(patsubst examples/webassembly/%.wat,examples/webassembly-bin/%.wasm,$(WAT))
WASM_HTML := $(patsubst examples/webassembly/%.wat,examples/webassembly-bin/%.wasm.html,$(WAT))

.PHONY: all
all: $(WASM) $(WASM_HTML)

.PHONY: clean
clean:
	$(and $(wildcard $(WASM) $(WASM_HTML)),rm $(wildcard $(WASM) $(WASM_HTML)))

$(WASM): examples/webassembly-bin/%.wasm: examples/webassembly/%.wat
	@mkdir -p $(dir $@)
	wat2wasm $^ -o $@

$(WASM_HTML): examples/webassembly-bin/%.wasm.html: examples/webassembly-bin/%.wasm examples/harness.html
	@mkdir -p $(dir $@)
	sed "s^{{file}}^data:application/wasm\;base64,$$(base64 -w 0 examples/webassembly-bin/$*.wasm)^" examples/harness.html > $@



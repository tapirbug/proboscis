WAT := $(wildcard examples/webassembly/*.wat)
WASM := $(patsubst examples/webassembly/%.wat,examples/webassembly-bin/%.wasm,$(WAT))
WASM_HTML := $(patsubst examples/webassembly/%.wat,examples/webassembly-bin/%.wasm.html,$(WAT))

TEST_LISP := $(wildcard test/lisp/*.lisp)
TEST_WAT := $(patsubst %.lisp,%.wat,$(TEST_LISP))
TEST_WASM := $(patsubst %.lisp,%.wasm,$(TEST_LISP))
TEST_HTML := $(patsubst %.lisp,%.html,$(TEST_LISP))

RUST_SRC := $(shell find src -name '*.rs')

CLEANABLE := $(WASM) $(WASM_HTML) $(TEST_WAT) $(TEST_WASM) $(TEST_HTML)

.PHONY: all
all: $(WASM) $(WASM_HTML) $(TEST_HTML)

.PHONY: clean
clean:
	$(and $(wildcard $(CLEANABLE)),rm $(wildcard $(CLEANABLE)))

$(WASM): examples/webassembly-bin/%.wasm: examples/webassembly/%.wat
	@mkdir -p $(dir $@)
	wat2wasm $^ -o $@

$(WASM_HTML): examples/webassembly-bin/%.wasm.html: examples/webassembly-bin/%.wasm examples/harness.html
	@mkdir -p $(dir $@)
	sed "s^{{file}}^data:application/wasm\;base64,$$(base64 -w 0 examples/webassembly-bin/$*.wasm)^" examples/harness.html > $@

$(TEST_WAT): %.wat: %.lisp $(RUST_SRC)
	cargo run -- $*.lisp -o $*.wat

$(TEST_WASM): %.wasm: %.wat
	wat2wasm $^ -o $@

$(TEST_HTML): %.html: %.wasm examples/harness.html
	sed "s^{{file}}^data:application/wasm\;base64,$$(base64 -w 0 $*.wasm)^" examples/harness.html > $@

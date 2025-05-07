TEST_LISP := $(wildcard test/lisp/*.lisp)
TEST_WAT := $(patsubst test/lisp/%.lisp,test/generated/%.wat,$(TEST_LISP))
TEST_WASM := $(patsubst %.wat,%.wasm,$(TEST_WAT))
TEST_HTML := $(patsubst %.wat,%.html,$(TEST_WAT))

RUST_SRC := $(shell find src -name '*.rs')

CLEANABLE := $(TEST_WAT) $(TEST_WASM) $(TEST_HTML)

.PHONY: all
all: $(TEST_WAT) $(TEST_WASM) $(TEST_HTML)

.PHONY: clean
clean:
	$(and $(wildcard $(CLEANABLE)),rm $(wildcard $(CLEANABLE)))

$(TEST_WAT): test/generated/%.wat: test/lisp/%.lisp $(RUST_SRC)
	@mkdir -p $(dir $@)
	cargo run -- test/lisp/$*.lisp -o test/generated/$*.wat

$(TEST_WASM): %.wasm: %.wat
	@mkdir -p $(dir $@)
	wat2wasm $^ -o $@

$(TEST_HTML): %.html: %.wasm test/harness.html
	@mkdir -p $(dir $@)
	sed "s^{{file}}^data:application/wasm\;base64,$$(base64 -w 0 $*.wasm)^" test/harness.html > $@

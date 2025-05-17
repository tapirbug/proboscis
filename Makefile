TEST_LISP := $(wildcard test/lisp/*.lisp)
TEST_WAT := $(patsubst test/lisp/%.lisp,test/generated/%.wat,$(TEST_LISP))
TEST_WASM := $(patsubst %.wat,%.wasm,$(TEST_WAT))
TEST_PIRT := $(patsubst %.wat,%.pirt,$(TEST_WAT))
TEST_HTML := $(patsubst %.wat,%.html,$(TEST_WAT))
PROBOSCIS := target/debug/proboscis

RUST_SRC := $(shell find src -name '*.rs')

CLEANABLE := $(TEST_WAT) $(TEST_WASM) $(TEST_HTML) $(TEST_PIRT)

.PHONY: all
all: $(TEST_WAT) $(TEST_WASM) $(TEST_PIRT) $(TEST_HTML)

.PHONY: clean
clean:
	$(and $(wildcard $(CLEANABLE)),rm $(wildcard $(CLEANABLE)))

$(PROBOSCIS): $(RUST_SRC)
	cargo build

$(TEST_WAT): test/generated/%.wat: test/lisp/%.lisp $(PROBOSCIS)
	@mkdir -p $(dir $@)
	$(PROBOSCIS) test/lisp/$*.lisp -o $@ -f wat

$(TEST_PIRT): test/generated/%.pirt: test/lisp/%.lisp $(PROBOSCIS)
	@mkdir -p $(dir $@)
	$(PROBOSCIS) test/lisp/$*.lisp -o $@ -f pirt

$(TEST_WASM): %.wasm: %.wat
	@mkdir -p $(dir $@)
	wat2wasm $^ -o $@

$(TEST_HTML): %.html: %.wasm test/harness.html
	@mkdir -p $(dir $@)
	sed "s^{{file}}^data:application/wasm\;base64,$$(base64 -w 0 $*.wasm)^" test/harness.html > $@

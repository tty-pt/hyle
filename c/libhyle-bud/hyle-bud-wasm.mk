# Single ownership for hyle-bud WASM sources (L03).
# Include this mk from any module that needs hyle-bud in WASM.
HYLE_BUD_WASM_ROOT ?= ../..
HYLE_BUD_WASM_SRC ?= $(HYLE_BUD_WASM_ROOT)/external/hyle/c/libhyle-bud/src/filter.c $(HYLE_BUD_WASM_ROOT)/external/hyle/c/libhyle-bud/src/table.c $(HYLE_BUD_WASM_ROOT)/external/hyle/c/libhyle-bud/src/picker.c $(HYLE_BUD_WASM_ROOT)/external/hyle/c/libhyle-bud/src/form.c
HYLE_BUD_WASM_CFLAGS ?= -I$(HYLE_BUD_WASM_ROOT)/external/hyle/c/libhyle-bud/include -I$(HYLE_BUD_WASM_ROOT)/external/hyle/include

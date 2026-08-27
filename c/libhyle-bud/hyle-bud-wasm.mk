# Single ownership for hyle-bud WASM sources (L03).
# Include this mk from any module that needs hyle-bud in WASM.
# Uses REPO_ROOT if defined, else falls back to relative detection.
HYLE_BUD_WASM_SRC ?= $(REPO_ROOT)/external/hyle/c/libhyle-bud/src/filter.c $(REPO_ROOT)/external/hyle/c/libhyle-bud/src/table.c $(REPO_ROOT)/external/hyle/c/libhyle-bud/src/picker.c
ifeq ($(REPO_ROOT),)
HYLE_BUD_WASM_SRC = ../../external/hyle/c/libhyle-bud/src/filter.c ../../external/hyle/c/libhyle-bud/src/table.c ../../external/hyle/c/libhyle-bud/src/picker.c
endif
HYLE_BUD_WASM_CFLAGS ?= -I$(REPO_ROOT)/external/hyle/c/libhyle-bud/include -I$(REPO_ROOT)/external/hyle/include
ifeq ($(REPO_ROOT),)
HYLE_BUD_WASM_CFLAGS = -I../../external/hyle/c/libhyle-bud/include -I../../external/hyle/include
endif

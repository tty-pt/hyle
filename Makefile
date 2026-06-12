FOLDER := hyle
all := libhyle hyle_test

LDLIBS-libhyle := -lqmap
libhyle-obj-y := src/ctx.o src/value.o src/query.o src/view.o src/field.o src/blueprint.o src/purify.o src/source.o
LDLIBS-hyle_test := -lhyle -lqmap

include ../mk/include.mk

# Also build a static archive for Rust FFI (avoids naming conflict with Rust cdylib)
lib/libhyle.a: ${libhyle-obj-y} lib
	ar rcs $@ ${libhyle-obj-y}

all: lib/libhyle.a # zig-bindings/libhyle_zig.a

# Zig bindings (native static lib + WASM module)
ZIG ?= zig
zig-bindings/libhyle_zig.a: zig-bindings/src/*.zig lib/libhyle.a
	cd zig-bindings && $(ZIG) build && cp zig-out/lib/libhyle_zig.a ../$@

zig-bindings/hyle_wasm.wasm: zig-bindings/src/*.zig
	cd zig-bindings && $(ZIG) build && cp zig-out/bin/hyle_wasm.wasm ../$@

zig-test:
	cd zig-bindings && $(ZIG) build test

test: all
	LD_LIBRARY_PATH=./lib:../libqmap/lib ./bin/hyle_test${EXE}
	$(MAKE) zig-test

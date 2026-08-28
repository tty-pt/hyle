FOLDER := hyle-source

all := libhyle-source

EXTRA_CFLAGS += -I../../include -I../../../libqmap/include -I../../../stoma/include
LDLIBS-libhyle-source := -lhyle -lqmap -lstoma -ljson-c

libhyle-source-obj-y := src/source_utils.o src/store_fs.o src/store_mem.o src/meta.o src/dsv.o src/json.o src/engine.o src/options.o

LDFLAGS-libhyle-source := -L../../lib -L../../../libqmap/lib -L../../../stoma/lib

include ../../../mk/include.mk

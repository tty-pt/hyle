FOLDER := hyle-bud

all := libhyle-bud

EXTRA_CFLAGS += -I../../include -I../libhyle-source/include
LDLIBS-libhyle-bud := -lhyle -lbud -lqmap -ljson-c -Wl,--no-as-needed -lhyle-source -Wl,--as-needed

libhyle-bud-obj-y := src/filter.o src/table.o src/picker.o src/form.o

LDFLAGS-libhyle-bud := -L../../lib -L../../../bud/lib -Wl,-rpath,$(shell cd ../../../bud/lib && pwd) -L../../../libqmap/lib -L../libhyle-source/lib -Wl,-rpath,$(shell cd ../libhyle-source/lib && pwd)

include ../../../mk/include.mk

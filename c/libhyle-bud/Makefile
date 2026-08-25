FOLDER := hyle-bud

all := libhyle-bud

LDLIBS-libhyle-bud := -lhyle -lbud -lqmap -ljson-c

libhyle-bud-obj-y := src/filter.o src/table.o src/picker.o

LDFLAGS-libhyle-bud := -L../../lib -L../../../bud/lib -L../../../libqmap/lib

include ../../../mk/include.mk

CC      = mips64-elf-gcc
LD      = mips64-elf-gcc
OBJCOPY = mips64-elf-objcopy
TARGET  = n64_ai.elf

SRC = $(wildcard src/*.c)
OBJ = $(SRC:.c=.o)

CFLAGS_COMMON = -G0 -nostdlib -ffreestanding -march=vr4300 -mabi=32 -Wall -Wextra -Wl,--gc-sections
LDFLAGS = -T linker.ld -nostdlib -Wl,--gc-sections

release: CFLAGS = -O2 $(CFLAGS_COMMON)
release: $(TARGET)

debug: CFLAGS = -O0 -g -fno-inline $(CFLAGS_COMMON)
debug: $(TARGET)

$(TARGET): $(OBJ)
	$(LD) $(CFLAGS) $(OBJ) $(LDFLAGS) -o $@
	size $@
	nm -n $@ > $(TARGET:.elf=.map)

clean:
	rm -f $(OBJ) $(TARGET) $(TARGET:.elf=.map)

.PHONY: release debug clean

CC = mips64-elf-gcc
CFLAGS = -Wall -O2 -G0 -march=vr4300 -mtune=vr4300 -mabi=32 -ffreestanding -nostdlib -mfix4300 -ffunction-sections -fdata-sections
LDFLAGS = -T linker.ld -ldragon -lnosys -lc -Wl,--gc-sections -mabi=32 -march=vr4300 -mtune=vr4300

TARGET = n64_ai_project.z64

SRC_DIR = src
OBJ_DIR = obj
SRCS = $(wildcard $(SRC_DIR)/*.c) 
OBJS = $(patsubst $(SRC_DIR)/%.c, $(OBJ_DIR)/%.o, $(SRCS))

all: $(TARGET)

$(OBJ_DIR):
	mkdir -p $(OBJ_DIR)

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -Iinclude -c $< -o $@

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) $(OBJS) $(LDFLAGS) -o $@

clean:
	rm -rf $(OBJ_DIR) $(TARGET)

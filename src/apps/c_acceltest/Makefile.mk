$(BUILD_APP_DIR)/c_acceltest.elf: $(call rwildcard,$(SRC_DIR)apps/c_acceltest/,*.c) $(BUILD_APP_DIR)/firestorm.o $(BUILD_APP_DIR)/tock.o $(BUILD_APP_DIR)/crt1.o $(BUILD_APP_DIR)/sys.o $(BUILD_APP_DIR)/arch.o $(APP_LIBC)
	@echo "Building $@"
	@$(CC) $(CFLAGS_BASE) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/c_acceltest/loader.ld -o $@ -ffreestanding -nostdlib -Wl,-Map=$(BUILD_APP_DIR)/app.Map $^
	@$(OBJDUMP) $(OBJDUMP_FLAGS) $@ > $(BUILD_APP_DIR)/app.lst

$(BUILD_APP_DIR)/c_acceltest.bin: $(BUILD_APP_DIR)/c_acceltest.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_APP_DIR)/c_acceltest.bin.o: $(BUILD_APP_DIR)/c_acceltest.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@


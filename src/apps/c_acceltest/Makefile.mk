$(BUILD_APP_DIR)/c_acceltest.elf: $(call rwildcard,$(SRC_DIR)apps/c_acceltest/,*.c) $(BUILD_DIR)/arch.o $(BUILD_APP_DIR)/firestorm.o $(BUILD_APP_DIR)/tock.o $(BUILD_APP_DIR)/crt1.o $(BUILD_APP_DIR)/sys.o $(APP_LIBC)
	@echo "Building $@"
	@$(CC) $(LDFLAGS) $(CFLAGS_APPS) -g -Os -T $(SRC_DIR)apps/c_acceltest/loader.ld -o $@ -ffreestanding -nostdlib $^

$(BUILD_APP_DIR)/c_acceltest.bin: $(BUILD_APP_DIR)/c_acceltest.elf
	@echo "Extracting binary $@"
	@$(OBJCOPY) --gap-fill 0xff -O binary $< $@ 

$(BUILD_APP_DIR)/c_acceltest.bin.o: $(BUILD_APP_DIR)/c_acceltest.bin
	@echo "Linking $@"
	@$(LD) -r -b binary -o $@ $<
	@$(OBJCOPY) --rename-section .data=.app.2 $@


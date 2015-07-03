
$(BUILD_DIR)/librf230.rlib: $(call rwildcard,src/chips/rf230,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libhil.rlib $(BUILD_DIR)/libcommon.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) src/chips/rf230/lib.rs

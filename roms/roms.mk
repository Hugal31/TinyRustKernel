include ../../config.mk

DEPS = $(OBJS:.o=.d)

MKKFS	= ../../tools/mkkfs/mkkfs

all: $(TARGET).rom

$(TARGET): CPPFLAGS += -MMD -I ../../libs/libc/include -I ../../libs/libk/include
$(TARGET): LDFLAGS += -Wl,-T../roms.lds
$(TARGET): LDLIBS = -L ../../libs/libk -L ../../libs/libc -Wl,--start-group -lc -lk -Wl,--end-group
$(TARGET): $(OBJS)

clean:
	$(RM) $(OBJS) $(DEPS) $(TARGET) $(TARGET).rom

$(TARGET).rom: $(TARGET)
	$(MKKFS) -o $@ -n $(ROM_TITLE) $(ROM_FILES)

-include $(DEPS)

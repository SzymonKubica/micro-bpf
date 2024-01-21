       
_esp_flash_mmap_prefetch_pad_size = 16;
_esp_memprot_prefetch_pad_size = 0;
_esp_memprot_align_size = 0;
MEMORY
{
  iram0_0_seg (RX) : org = 0x40080000, len = 0x20000
  iram0_2_seg (RX) : org = 0x400D0020, len = 0x330000-0x20
  dram0_0_seg (RW) : org = 0x3FFB0000 + 0,
                                     len = 0x2c200 - 0
  drom0_0_seg (R) : org = 0x3F400020, len = 0x400000-0x20
  rtc_iram_seg(RWX) : org = 0x400C0000, len = 0x2000
  rtc_data_seg(RW) : org = 0x3ff80000, len = 0x2000 - 0
  rtc_slow_seg(RW) : org = 0x50000000 + 0,
                                     len = 0x2000 - 0
  extern_ram_seg(RWX) : org = 0x3F800000,
                                     len = 0x400000
}
_static_data_end = _bss_end;
_heap_end = 0x40000000 - 0x0;
_data_seg_org = ORIGIN(rtc_data_seg);
REGION_ALIAS("rtc_data_location", rtc_slow_seg );
  REGION_ALIAS("default_code_seg", iram0_2_seg);
  REGION_ALIAS("default_rodata_seg", drom0_0_seg);
  ASSERT(_rodata_start == ORIGIN(default_rodata_seg),
         ".flash.appdesc section must be placed at the beginning of the rodata segment.")

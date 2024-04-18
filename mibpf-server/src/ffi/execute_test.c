#include <stdio.h>
#include <string.h>
#include <stdint.h>

__attribute__((aligned(4))) unsigned char code[] = {
  0x4f, 0xf0, 0x2a, 0x00, 0x70, 0x47
};

int test(void) {
    union {
      uintptr_t as_int;
      int(*fn)(void);
    } helper;

    helper.as_int = ((uintptr_t)&code[0] | 0x1);

    int i = helper.fn();

    printf("get this done. returned: %d\n", i);

    return 0;
}

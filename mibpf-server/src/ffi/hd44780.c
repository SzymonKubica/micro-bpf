#include <stdio.h>

#include "hd44780.h"
#include "hd44780_params.h"

hd44780_t dev;

// Initializes the HD44780 display. Given that there is only one display connected
// to the device, this should be called once at the start of the main function
// (or lazily when the display is first used) and then all components that want
// to print something to the display should be given a singleton handle that
// contains the pointer to the device struct defined above.
int32_t hd44780_init_default(void)
{
    if (hd44780_init(&dev, &hd44780_params[0]) != 0) {
        puts("[FAILED]");
        return -1;
    };
    return (int32_t)&dev;
}

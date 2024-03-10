#include <stdint.h>
#include <linux/bpf.h>
#include <math.h>
#include "../helpers.h"

#define D7_PORT 5
#define D7_PIN 13

#define D7_PORT 5
#define D7_PIN 13

#define D8_PORT 5
#define D8_PIN 12

#define D5_PORT 4
#define D5_PIN 11

#define D11_PORT 0
#define D11_PIN 6

// The LEDs are connected in a way that they are always connected to 5V on
// one end the other end is connected to the digital GPIO, we turn the leds on
// by turning the GPIO off.
#define ON 0
#define OFF 4096

// Refresh period of the LED status
#define PERIOD_US (250 * 1000)

#define TEMPERATURE_STORAGE_INDEX 0
#define HUMIDITY_STORAGE_INDEX 1

void set_led(uint32_t index, uint32_t value);
void handle_temperature_data(uint16_t temp);
void handle_humidity_data(uint16_t hum);


int set_led_given_dht_data(void *ctx)
{
    (void)ctx;

    uint32_t last_wakeup = bpf_ztimer_now();
    while (1) {
        bpf_ztimer_periodic_wakeup(&last_wakeup, PERIOD_US);

        int temp = 0;
        int hum = 0;

        bpf_fetch_global(TEMPERATURE_STORAGE_INDEX, &temp);
        bpf_fetch_global(HUMIDITY_STORAGE_INDEX, &hum);

        // TODO: investigate why the printf fails here with out of bounds access
        //bpf_printf("[led controller] retrieved temp: %d.%d°C\n", temp / 10, temp % 10);
        // bpf_printf("[led controller] retrieved relative humidity: %d.%d%%\n", hum / 10, hum % 10);

        handle_temperature_data(temp);
        handle_humidity_data(hum);

    }
    return 0;
}

void handle_temperature_data(uint16_t temp)
{
    if (temp > 250) {
        set_led(2, ON);
        set_led(1, OFF);
        set_led(0, OFF);
    } else if (temp > 200) {
        set_led(2, OFF);
        set_led(1, ON);
        set_led(0, OFF);
    } else {
        set_led(2, OFF);
        set_led(1, OFF);
        set_led(0, ON);
    }
}

void handle_humidity_data(uint16_t hum)
{
    // Humidity is given by a percentage with one decimal point
    // so 80% is represented as 800
    // The led 3 is wired pull down and so to turn in on we need to write
    // to the pin high
    if (hum > 800) {
        set_led(3, 4096);
    } else {
        set_led(3, 0);
    }
}

void set_led(uint32_t index, uint32_t value)
{
    switch (index) {
    case 0:
        bpf_gpio_write(D5_PORT, D5_PIN, value);
        break;
    case 1:
        bpf_gpio_write(D7_PORT, D7_PIN, value);
        break;
    case 2:
        bpf_gpio_write(D8_PORT, D8_PIN, value);
        break;
    case 3:
        bpf_gpio_write(D11_PORT, D11_PIN, value);
        break;
    }
}

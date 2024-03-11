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
#define PERIOD_US (1000 * 1000)

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

        handle_temperature_data(temp);
        handle_humidity_data(hum);

    }
    return 0;
}

void handle_temperature_data(uint16_t temp)
{
    if (temp > 250) {
        bpf_printf("Temperature above 25C detected, toggling Warning LED\n");
    } else if (temp > 200) {
        bpf_printf("Temperature between 20-25C detected, toggling Normal LED\n");
    } else {
        bpf_printf("Temperature below 20C detected, toggling TooLow LED\n");
    }
}

void handle_humidity_data(uint16_t hum)
{
    // Humidity is given by a percentage with one decimal point
    // so 80% is represented as 800
    // The led 3 is wired pull down and so to turn in on we need to write
    // to the pin high
    if (hum > 800) {
        bpf_printf("Humidity above 80%% detected, toggling rain indicator LED\n");
    }
}


#include <stdint.h>
#include "helpers.h"
#include <stdbool.h>
#include "constants.h"

// Keypad button readings
#define RIGHT 0
#define UP 1
#define DOWN 2
#define LEFT 3
#define NO_INPUT 4

#define BUTTON_POLLING_PERIOD (25 * 1000) // 250 ms -> human reaction time
#define DISPLAY_BUTTONS_ADC 2

#define DISPLAY_UPDATE_PERIOD (4 * 50) // Every 5 seconds

char temperature_fmt[] = "Temperature";
char humidity_fmt[] = "Humidity";
char light_intensity_fmt[] = "Light Intensity: ";
char sound_intensity_fmt[] = "Sound Intensity: ";

#define TEMPERATURE 0
#define HUMIDITY 1
#define LIGHT_INTENSITY 2
#define SOUND_INTENSITY 3

int lcd_display_measurement_logging(void *ctx)
{
        uint32_t start = bpf_ztimer_now();
        uint64_t dev = bpf_hd44780_init();
        bpf_hd44780_clear(dev);

        uint32_t counter = 0;
        uint32_t current_measurement = TEMPERATURE;
        uint32_t previous_input = NO_INPUT;

        while (1) {
                counter = (counter + 1) % DISPLAY_UPDATE_PERIOD;
                bpf_ztimer_periodic_wakeup(&start, BUTTON_POLLING_PERIOD);
                uint32_t new_input = bpf_keypad_get_input(DISPLAY_BUTTONS_ADC);
                bool update_display = false;
                if (new_input != previous_input) {
                        if (new_input == UP) {
                                current_measurement =
                                    (current_measurement + 1) % 4;
                                update_display = true;
                        }
                        if (new_input == DOWN) {
                                current_measurement =
                                    (current_measurement - 1) % 4;
                                update_display = true;
                        }
                        if (new_input == RIGHT) {
                            bpf_hd44780_clear(dev);
                            bpf_hd44780_set_cursor(dev, 0, 0);
                            bpf_hd44780_print(dev, "Error");
                            int32_t *invalid_address = (int32_t*)-1;
                            uint32_t invalid_value = *invalid_address;
                            bpf_printf("Invalid memory access value: %d\n", invalid_value);
                        }
                        previous_input = new_input;
                }
                if (!update_display && counter % DISPLAY_UPDATE_PERIOD == 0) {
                        current_measurement = (current_measurement + 1) % 4;
                        update_display = true;
                }

                if (update_display) {
                        bpf_hd44780_clear(dev);
                        bpf_hd44780_set_cursor(dev, 0, 0);
                        if (current_measurement == TEMPERATURE) {
                                uint32_t temperature;
                                bpf_hd44780_print(dev, "Temperature: ");
                                bpf_fetch_global(DHT1_TEMP_STORAGE_INDEX,
                                                 &temperature);
                                char fmt_buffer[5] = "    ";
                                bpf_hd44780_set_cursor(dev, 0, 1);
                                size_t str_len = bpf_fmt_s16_dfp(
                                    fmt_buffer, temperature, -1);
                                bpf_hd44780_print(dev, fmt_buffer);
                                bpf_hd44780_print(dev, "C");
                        } else if (current_measurement == HUMIDITY) {
                                bpf_hd44780_print(dev, "Humidity: ");
                                uint32_t humidity;
                                bpf_fetch_global(DHT1_HUM_STORAGE_INDEX,
                                                 &humidity);
                                char fmt_buffer[5] = "    ";
                                bpf_hd44780_set_cursor(dev, 0, 1);
                                size_t str_len =
                                    bpf_fmt_s16_dfp(fmt_buffer, humidity, -1);
                                bpf_hd44780_print(dev, fmt_buffer);
                                bpf_hd44780_print(dev, "%");

                        } else if (current_measurement == LIGHT_INTENSITY) {
                                bpf_hd44780_print(dev, light_intensity_fmt);
                                uint32_t light_intensity;
                                bpf_fetch_global(LIGHT_INTENSITY_STORAGE_INDEX,
                                                 &light_intensity);
                                char fmt_buffer[3];
                                bpf_hd44780_set_cursor(dev, 0, 1);
                                size_t str_len = bpf_fmt_u32_dec(
                                    fmt_buffer, light_intensity);
                                bpf_hd44780_print(dev, fmt_buffer);
                                bpf_hd44780_print(dev, "%");
                        } else if (current_measurement == SOUND_INTENSITY) {
                                bpf_hd44780_print(dev, sound_intensity_fmt);
                                uint32_t sound_intensity;
                                bpf_fetch_global(SOUND_INTENSITY_STORAGE_INDEX,
                                                 &sound_intensity);
                                char fmt_buffer[3];
                                bpf_hd44780_set_cursor(dev, 0, 1);
                                size_t str_len = bpf_fmt_u32_dec(
                                    fmt_buffer, sound_intensity);
                                bpf_hd44780_print(dev, fmt_buffer);
                                bpf_hd44780_print(dev, "dB");
                        }
                        // We wait after printing not to mess up the display
                        bpf_ztimer_periodic_wakeup(&start,
                                                   10 * BUTTON_POLLING_PERIOD);
                }
        }

        // Unreachable
        return 0;
}

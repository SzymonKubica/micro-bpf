#include <stdio.h>

#include "dht.h"
#include "dht_params.h"
#include "fmt.h"
#include "ztimer.h"

// The DHT sensor is connected to D2 which corresponds to PA_10
#define PORT 5 // Port A
#define PIN 15

#define ENABLE_DEBUG (1)

void dht_test_read(dht_t *dev)
{
    int16_t temp, hum;
    int ret = dht_read(dev, &temp, &hum);
    if (ret == DHT_OK) {
        printf("DHT sensor connected\n");
    } else if (ret == -ENODEV) {
        printf("Sensor didn't respond to read request\n");
        return;
    } else if (ret == -EIO) {
        printf("Received and expected checksums don't match\n");
        return;
    } else if (ret == -ENOSYS) {
        printf("Unable to parse the received data\n");
        return;
    } else if (ret == -ERANGE) {
        printf("Misconfigured device\n");
        return;
    } else {
        printf("Unknown error: %d\n", ret);
        return;
    }
    char temp_s[10];
    size_t n = fmt_s16_dfp(temp_s, temp, -1);
    temp_s[n] = '\0';

    char hum_s[10];
    n = fmt_s16_dfp(hum_s, hum, -1);
    hum_s[n] = '\0';
    printf("DHT values - temp: %s°C - relative humidity: %s%%\n", temp_s,
           hum_s);
}

int dht_test(void)
{
    dht_params_t my_params;
    my_params.pin = GPIO_PIN(PORT, PIN);
    my_params.type = DHT22;
    my_params.in_mode = DHT_PARAM_PULL;

    dht_t dev;
    int ret = dht_init(&dev, &my_params);
    if (ret == DHT_OK) {
        printf("DHT sensor connected\n");
    } else if (ret == -ENODEV) {
        printf("Sensor didn't respond\n");
        return 1;
    } else if (ret == -EXDEV) {
        printf("Invalid cross-device link\n");
        return 1;
    } else {
        printf("Unknown error: %d\n", ret);
        return 1;
    }

    ztimer_sleep(ZTIMER_MSEC, 2000);
    dht_test_read(&dev);
    return 0;
}

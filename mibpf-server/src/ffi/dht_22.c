#include <stdio.h>

#include "dht.h"
#include "dht_params.h"
#include "fmt.h"
#include "time_units.h"
#include "ztimer.h"

#define DELAY (2 * US_PER_SEC)

int dht_test(void)
{
    dht_t dev;
    int16_t temp, hum;

    puts("DHT temperature and humidity sensor test application\n");

    /* initialize first configured sensor */
    printf("Initializing DHT sensor...\t");
    if (dht_init(&dev, &dht_params[0]) == DHT_OK) {
        puts("[OK]\n");
    } else {
        puts("[Failed]");
        return 1;
    }

    ztimer_sleep(ZTIMER_USEC, DELAY);

    if (dht_read(&dev, &temp, &hum) != DHT_OK) {
        puts("Error reading values");
        return -1;
    }

    printf("DHT values - temp: %d.%d°C - relative humidity: %d.%d%%\n",
           temp / 10, temp % 10, hum / 10, hum % 10);

    return 0;
}

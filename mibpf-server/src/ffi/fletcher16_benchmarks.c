#include <stdint.h>
#include <string.h>

// A random 80B string
const char DATA_80B[] = "kMWChVp8hmuPtBMC9jRxQfgySeaVORRXOYu6p1am"
                        "GVkdOz1Yxkxo4wL8tofWBjpqI39rb9g3Q0lssuY6";
// A random 160B string
const char DATA_160B[] =
    "s47fVMWNCMUy2Lw46PefYi0uBoYtHGXWZO43VtUSzMumyVqm9Prefxk3iFMoJ07pjVemTg"
    "d0ntYjWmDDYocrpJQ8LcMpEBbs3pIBAKnT8z729PEwmKGhxd8YDysBLqtWSqImirpXQcC7"
    "KniYzrRRSebNEeJdXHLo";

// A random 320B string
const char DATA_320B[] =
    "46WgyN33S3oADXpVVIMX1ki2aMcO7fi8SN5HqvDtSJ6jqA96oHAKptpcAyxhVk4y"
    "2qlIEQB4YqErDyXUwMVJnOJEFzrHT0MC2RuOcY9tLCImE7OXyAU7opoXfKmkw8e6"
    "Q8Qm6wNAD7DHsBLYexQzXe2WDwADWaz6mENTwXqF6ZecRo2IyU9u93KFD3meVeIC"
    "fDezW9OeqLIDjwQ7FnOGwjSEeCZAqlpXACKmw3G2lsMHhGm44pygbapiYvBrfCgG"
    "UBNLhGdlUt9Hk0dCuBwAZjLu0pAf0ddJNicky8dUT9Zo6JNKkbrfuTU6cCfHe2nQ";

// A random 640B string
const char DATA_640B[] =
    "46WgyN33S3oADXpVVIMX1ki2aMcO7fi8SN5HqvDtSJ6jqA96oHAKptpcAyxhVk4y"
    "2qlIEQB4YqErDyXUwMVJnOJEFzrHT0MC2RuOcY9tLCImE7OXyAU7opoXfKmkw8e6"
    "Q8Qm6wNAD7DHsBLYexQzXe2WDwADWaz6mENTwXqF6ZecRo2IyU9u93KFD3meVeIC"
    "fDezW9OeqLIDjwQ7FnOGwjSEeCZAqlpXACKmw3G2lsMHhGm44pygbapiYvBrfCgG"
    "UBNLhGdlUt9Hk0dCuBwAZjLu0pAf0ddJNicky8dUT9Zo6JNKkbrfuTU6cCfHe2nQ"
    "vZKGgfVQPuqoz4ahGJthZjUWsdXzREJSHmJIWvnFmarMd84mPQNKKqTH4kJMmy8c"
    "TMDyl5Gf81oscb2yFV7O8JizXETfnuvx5p0UqfzPr7E5AkRnbTd4m8135Vo4oVNH"
    "iKCE2HAdS6KPUUPLeMIJm7JfMx1a1bkchrkzu9EkO9CuYrPGsN2CMRIxuckPpK2q"
    "Iys8mm3oayC1z1sjZdboQDNR9oENO509932Zz0hA1ZjVWUvVzWc9cBbtabIFlHCs"
    "kkhKsjyydGJ6bddk0gSLzcseoGsaWgOfVhN4K9oysNZbs469FAdPBoTukToFaz7K";
// A random 1280B string
const char DATA_1280B[] =
    "46WgyN33S3oADXpVVIMX1ki2aMcO7fi8SN5HqvDtSJ6jqA96oHAKptpcAyxhVk4y"
    "2qlIEQB4YqErDyXUwMVJnOJEFzrHT0MC2RuOcY9tLCImE7OXyAU7opoXfKmkw8e6"
    "Q8Qm6wNAD7DHsBLYexQzXe2WDwADWaz6mENTwXqF6ZecRo2IyU9u93KFD3meVeIC"
    "fDezW9OeqLIDjwQ7FnOGwjSEeCZAqlpXACKmw3G2lsMHhGm44pygbapiYvBrfCgG"
    "UBNLhGdlUt9Hk0dCuBwAZjLu0pAf0ddJNicky8dUT9Zo6JNKkbrfuTU6cCfHe2nQ"
    "vZKGgfVQPuqoz4ahGJthZjUWsdXzREJSHmJIWvnFmarMd84mPQNKKqTH4kJMmy8c"
    "TMDyl5Gf81oscb2yFV7O8JizXETfnuvx5p0UqfzPr7E5AkRnbTd4m8135Vo4oVNH"
    "iKCE2HAdS6KPUUPLeMIJm7JfMx1a1bkchrkzu9EkO9CuYrPGsN2CMRIxuckPpK2q"
    "Iys8mm3oayC1z1sjZdboQDNR9oENO509932Zz0hA1ZjVWUvVzWc9cBbtabIFlHCs"
    "kkhKsjyydGJ6bddk0gSLzcseoGsaWgOfVhN4K9oysNZbs469FAdPBoTukToFaz7K"
    "46WgyN33S3oADXpVVIMX1ki2aMcO7fi8SN5HqvDtSJ6jqA96oHAKptpcAyxhVk4y"
    "2qlIEQB4YqErDyXUwMVJnOJEFzrHT0MC2RuOcY9tLCImE7OXyAU7opoXfKmkw8e6"
    "Q8Qm6wNAD7DHsBLYexQzXe2WDwADWaz6mENTwXqF6ZecRo2IyU9u93KFD3meVeIC"
    "fDezW9OeqLIDjwQ7FnOGwjSEeCZAqlpXACKmw3G2lsMHhGm44pygbapiYvBrfCgG"
    "UBNLhGdlUt9Hk0dCuBwAZjLu0pAf0ddJNicky8dUT9Zo6JNKkbrfuTU6cCfHe2nQ"
    "vZKGgfVQPuqoz4ahGJthZjUWsdXzREJSHmJIWvnFmarMd84mPQNKKqTH4kJMmy8c"
    "TMDyl5Gf81oscb2yFV7O8JizXETfnuvx5p0UqfzPr7E5AkRnbTd4m8135Vo4oVNH"
    "iKCE2HAdS6KPUUPLeMIJm7JfMx1a1bkchrkzu9EkO9CuYrPGsN2CMRIxuckPpK2q"
    "Iys8mm3oayC1z1sjZdboQDNR9oENO509932Zz0hA1ZjVWUvVzWc9cBbtabIFlHCs"
    "kkhKsjyydGJ6bddk0gSLzcseoGsaWgOfVhN4K9oysNZbs469FAdPBoTukToFaz7K";

// A random 2560B string
const char DATA_2560B[] =
    "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";

static inline uint32_t fletcher16(char *data);

uint32_t fletcher_16_80B(void) { return fletcher16(DATA_80B); }
uint32_t fletcher_16_160B(void) { return fletcher16(DATA_160B); }
uint32_t fletcher_16_320B(void) { return fletcher16(DATA_320B); }
uint32_t fletcher_16_640B(void) { return fletcher16(DATA_640B); }
uint32_t fletcher_16_1280B(void) { return fletcher16(DATA_1280B); }
uint32_t fletcher_16_2560B(void) { return fletcher16(DATA_2560B); }

static inline uint32_t fletcher16(char *data)
{
    uint8_t *data_ptr = (uint8_t *)data;

    int len = (strlen(data) + 1) & ~1; /* Round up len to words */

    uint16_t sum1 = 0;
    uint16_t sum2 = 0;
    int index;

    for (index = 0; index < len; ++index) {
        sum1 = (sum1 + data_ptr[index]) % 255;
        sum2 = (sum2 + sum1) % 255;
    }

    return (sum2 << 8) | sum1;
}

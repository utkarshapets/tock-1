#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"

// Pin definitions
#define LED_0 0

#ifdef __cplusplus
extern "C" {
#endif

enum firestorm_cb_type {
  PUTSTR,
  READTMP,
  READACCEL,
  READMAGNET,
  SPIBUF,
  ASYNC
};

int gpio_enable(unsigned int pin);
int gpio_set(unsigned int pin);
int gpio_clear(unsigned int pin);
int gpio_toggle(unsigned int pin);

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

int timer_oneshot_subscribe(subscribe_cb cb, void *userdata);
int timer_repeating_subscribe(subscribe_cb cb, void *userdata);


int spi_read_write(const char* write, char* read, size_t  len, subscribe_cb cb);

typedef struct accel_result {
    int16_t x;
    int16_t y;
    int16_t z;
} accel_result_t;

typedef struct magnet_result {
    int16_t x;
    int16_t y;
    int16_t z;
} magnet_result_t;

int FXOS8700CQ_accel_enable();
int FXOS8700CQ_accel_read(accel_result_t *res);
int FXOS8700CQ_accel_read_async(subscribe_cb cb, void* userdata);
int FXOS8700CQ_magnet_enable();
int FXOS8700CQ_magnet_read(magnet_result_t *res);
int FXOS8700CQ_magnet_read_async(subscribe_cb cb, void* userdata);

#ifdef __cplusplus
}
#endif

#endif // _FIRESTORM_H

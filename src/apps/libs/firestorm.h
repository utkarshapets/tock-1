#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

enum firestorm_cb_type {
  PUTSTR,
  READTMP,
  READACCEL,
  READMAGNET,
  ASYNC
};

int gpio_enable(unsigned int pin);
int gpio_set(unsigned int pin);
int gpio_clear(unsigned int pin);

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

int tmp006_enable();
int tmp006_read(int16_t *temperature);
int tmp006_read_async(subscribe_cb cb, void* userdata);

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

#ifndef _fxos8700cq_h
#define _fxos8700cq_h

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define ERR_NONE 0

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
int FXOS8700CQ_accel_read_sync(accel_result_t *res);
int FXOS8700CQ_accel_read_async(subscribe_cb cb, void* userdata);
int FXOS8700CQ_magnet_enable();
int FXOS8700CQ_magnet_read_sync(magnet_result_t *res);
int FXOS8700CQ_magnet_read_async(subscribe_cb cb, void* userdata);


#ifdef __cplusplus
}
#endif

#endif // _fxos8700cq_h

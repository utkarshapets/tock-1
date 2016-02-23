#ifndef _TOCK_H
#define _TOCK_H

#include <inttypes.h>
#include <unistd.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct driver_num {
  uint32_t num;
} driver_t;

typedef int CB_TYPE;

typedef CB_TYPE (subscribe_cb)(int, int, int,void*);

CB_TYPE wait();
CB_TYPE wait_for();
int command(driver_t driver, uint32_t command, int data);
int subscribe(driver_t, uint32_t subscribe,
              subscribe_cb cb, void* userdata);
int allow(driver_t driver, uint32_t allow, void* ptr, size_t size);

#ifdef __cplusplus
}
#endif

#endif // _TOCK_H

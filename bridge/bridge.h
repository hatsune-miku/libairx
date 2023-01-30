#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define LENGTH_PRESERVE_SIZE 16

#define MESSAGE_MAX_SIZE ((2 << (LENGTH_PRESERVE_SIZE - 1)) - 1)

typedef struct AirXService AirXService;

int32_t airx_version(void);

bool airx_is_first_run(void);

struct AirXService *airx_create(uint16_t discovery_service_server_port,
                                uint16_t discovery_service_client_port,
                                char *text_service_listen_addr,
                                uint16_t text_service_listen_port);

struct AirXService *airx_restore(void);

void airx_lan_discovery_service(struct AirXService *airx_ptr);

void airx_lan_discovery_service_async(struct AirXService *airx_ptr);

void airx_text_service(struct AirXService *airx_ptr);

void airx_text_service_async(struct AirXService *airx_ptr);

bool airx_lan_broadcast(struct AirXService *airx_ptr);

uint32_t airx_get_peers(struct AirXService *airx_ptr, char *buffer);

void airx_start_auto_broadcast(struct AirXService *airx_ptr);

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define PACKET_SIZE 12

/**
 *  * Serialized as:     * 2 bytes: magic number     * 4 bytes: data length in bytes     * N bytes: data     * 2 bytes: hash of (data_length)     * 8 + N bytes in total
 */
#define BASE_PACKET_SIZE 8

#define STRING_LENGTH_MAX 65535

typedef struct AirXService AirXService;

int32_t airx_version(void);

int32_t airx_compatibility_number(void);

void airx_init(void);

struct AirXService *airx_create(uint16_t discovery_service_server_port,
                                uint16_t discovery_service_client_port,
                                char *text_service_listen_addr,
                                uint32_t text_service_listen_addr_len,
                                uint16_t text_service_listen_port,
                                uint8_t group_identity);

void airx_lan_discovery_service(struct AirXService *airx_ptr, bool (*should_interrupt)(void));

void airx_text_service(struct AirXService *airx_ptr,
                       void (*text_callback_c)(const char *, uint32_t, const char *, uint32_t),
                       void (*file_coming_callback_c)(uint32_t, const char *, uint32_t, const char *, uint32_t),
                       bool (*should_interrupt)(void));

bool airx_lan_broadcast(struct AirXService *airx_ptr);

uint32_t airx_get_peers(struct AirXService *airx_ptr, char *buffer);

void airx_send_text(struct AirXService *airx_ptr,
                    const char *host,
                    uint32_t host_len,
                    char *text,
                    uint32_t text_len);

void airx_broadcast_text(struct AirXService *airx_ptr, char *text, uint32_t len);

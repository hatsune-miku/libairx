#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct AirXService AirXService;

int32_t airx_version(void);

int32_t airx_compatibility_number(void);

uint64_t airx_version_string(char *buffer);

void airx_init(void);

struct AirXService *airx_create(uint16_t discovery_service_server_port,
                                uint16_t discovery_service_client_port,
                                char *text_service_listen_addr,
                                uint32_t text_service_listen_addr_len,
                                uint16_t text_service_listen_port,
                                uint32_t group_identifier);

void airx_lan_discovery_service(struct AirXService *airx_ptr, bool (*should_interrupt)(void));

void airx_data_service(struct AirXService *airx_ptr,
                       void (*text_callback_c)(const char*, uint32_t, const char*, uint32_t),
                       void (*file_coming_callback_c)(uint64_t, const char*, uint32_t, const char*, uint32_t),
                       void (*file_sending_callback_c)(uint8_t, uint64_t, uint64_t, uint8_t),
                       bool (*file_part_callback_c)(uint8_t, uint64_t, uint64_t, const uint8_t*),
                       bool (*should_interrupt)(void));

bool airx_lan_broadcast(struct AirXService *airx_ptr);

uint32_t airx_get_peers(struct AirXService *airx_ptr, char *buffer);

void airx_send_text(struct AirXService *airx_ptr,
                    const char *host,
                    uint32_t host_len,
                    char *text,
                    uint32_t text_len);

void airx_broadcast_text(struct AirXService *airx_ptr, char *text, uint32_t len);

void airx_try_send_file(struct AirXService *airx_ptr,
                        const char *host,
                        uint32_t host_len,
                        const char *file_path,
                        uint32_t file_path_len);

void airx_respond_to_file(struct AirXService *airx_ptr,
                          const char *host,
                          uint32_t host_len,
                          uint8_t file_id,
                          uint64_t file_size,
                          const char *file_path,
                          uint32_t file_path_len,
                          bool accept);

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

constexpr static const uintptr_t LENGTH_PRESERVE_SIZE = 16;

constexpr static const uintptr_t MESSAGE_MAX_SIZE = ((2 << (LENGTH_PRESERVE_SIZE - 1)) - 1);

struct AirXService;

extern "C" {

int32_t airx_version();

AirXService *airx_create(uint16_t discovery_service_server_port,
                         uint16_t discovery_service_client_port,
                         char *text_service_listen_addr,
                         uint16_t text_service_listen_port);

void airx_lan_discovery_service(AirXService *airx_ptr);

void airx_text_service(AirXService *airx_ptr);

bool airx_lan_broadcast(AirXService *airx_ptr);

uintptr_t airx_get_peers(AirXService *airx_ptr, char *buffer);

} // extern "C"

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class ClientError {
  ClientErrorNone,
  ClientErrorConnectFailure,
  ClientErrorNotConnected,
  ClientErrorStreamEnd,
  ClientErrorEnd,
};

enum class DeviceEventKindTag {
  TrackingEvent,
  ImpactEvent,
  ConnectEvent,
  DisconnectEvent,
};

enum class DeviceTag {
  Udp,
  Hid,
  Cdc,
};

enum class EventTag {
  None,
  DeviceEvent,
};

template<typename T = void>
struct Box;

struct Client;

struct Handle;

struct UserObj {
  const void *_0;
};

struct SocketAddr {
  const char *ip;
  uint16_t port;
};

struct UdpDevice {
  uint8_t id;
  SocketAddr addr;
  uint8_t uuid[6];
};

struct HidDevice {
  const char *path;
  uint8_t uuid[6];
};

struct CdcDevice {
  const char *path;
  uint8_t uuid[6];
};

union DeviceU {
  UdpDevice udp;
  HidDevice hid;
  CdcDevice cdc;
};

struct Device {
  DeviceTag tag;
  DeviceU u;
};

struct Vector2f64 {
  double x;
  double y;
};

struct Matrix3f64 {
  double m11;
  double m12;
  double m13;
  double m21;
  double m22;
  double m23;
  double m31;
  double m32;
  double m33;
};

struct Matrix3x1f64 {
  double x;
  double y;
  double z;
};

struct Pose {
  Matrix3f64 rotation;
  Matrix3x1f64 translation;
};

struct TrackingEvent {
  uint32_t timestamp;
  Vector2f64 aimpoint;
  Pose pose;
  bool pose_resolved;
  uint32_t screen_id;
};

struct ImpactEvent {
  uint32_t timestamp;
};

struct ConnectEvent {
  uint8_t _unused;
};

struct DisconnectEvent {
  uint8_t _unused;
};

union DeviceEventKindU {
  TrackingEvent tracking_event;
  ImpactEvent impact_event;
  ConnectEvent connect_event;
  DisconnectEvent disconnect_event;
};

struct DeviceEventKind {
  DeviceEventKindTag tag;
  DeviceEventKindU u;
};

struct DeviceEvent {
  Device device;
  DeviceEventKind kind;
};

union EventU {
  uint8_t none;
  DeviceEvent device_event;
};

struct Event {
  EventTag tag;
  EventU u;
};

extern "C" {

Box<Handle> init();

void free(Handle *handle);

Box<Client> client_new();

void client_connect(const Handle *handle,
                    UserObj userdata,
                    Client *client,
                    void (*callback)(UserObj userdata, ClientError error));

void client_get_device_list(const Handle *handle,
                            UserObj userdata,
                            Client *client,
                            void (*callback)(UserObj userdata,
                                             ClientError error,
                                             Device *device_list,
                                             uintptr_t size));

void start_stream(const Handle *handle,
                  UserObj userdata,
                  Client *client,
                  void (*callback)(UserObj userdata, ClientError error, Event reply));

} // extern "C"

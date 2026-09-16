# ---------------------------------------------------------------------------
# QuYou discovery protocol
# ---------------------------------------------------------------------------
#
# The QuYou app discovers Qu mixers using UDP broadcast on port 51320.
#
# REQUEST:
#   - Transport: UDP
#   - Destination: 255.255.255.255:51320
#   - Source: QuYou's local IP and an ephemeral UDP source port
#   - Payload: 7 bytes
#   - Payload: b"QU Find"
#   - Hex: 51 55 20 46 69 6e 64
#
#   QuYou sends the discovery request approximately once per second.
#   The source port is ephemeral and may change when discovery is restarted,
#   for example when Refresh is pressed in QuYou.
#
# RESPONSE:
#   - Transport: UDP
#   - Source: Qu mixer IP:51320
#   - Destination: the source IP and source port of the discovery request
#   - Payload: device name followed by a NUL byte
#   - Qu-16 payload: 6 bytes
#   - Qu-16 payload: b"Qu-16\x00"
#   - Qu-16 hex: 51 75 2d 31 36 00
#
#   The response is unicast rather than broadcast. The Qu sends it directly
#   back to the IP address and UDP source port from which the request arrived.
#
# Example exchange:
#
#   QuYou -> 255.255.255.255:51320
#       51 55 20 46 69 6e 64
#       "QU Find"
#
#   Qu-16 -> QuYou:<source-port>
#       51 75 2d 31 36 00
#       "Qu-16\0"
#
# On Linux, the client's UDP response port must be permitted through the
# local firewall. For example, with UFW:
#
#   sudo ufw allow 51321/udp
#
# Wireshark filter for discovery traffic:
#
#   udp.port == 51320
#
# To isolate discovery requests specifically:
#
#   udp.dstport == 51320
#
# To isolate responses from a Qu:
#
#   ip.src == <qu-ip> && udp.srcport == 51320
#   NB. as this is unicast, use Wireshark on the device making the request!
#
# ---------------------------------------------------------------------------
import socket


DISCOVERY_PORT = 51320
DISCOVERY_CLIENT_PORT = 51321
DISCOVERY_MESSAGE = b"QU Find"


sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

# Allow broadcast transmission.
sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)

# Bind to a fixed client-side port before sending.
sock.bind(("0.0.0.0", DISCOVERY_CLIENT_PORT))

# Wait for responses for three seconds.
sock.settimeout(3)

local_ip, local_port = sock.getsockname()

print(f"Listening on {local_ip}:{local_port}")

# Send the discovery request using the same socket/source port.
sock.sendto(
    DISCOVERY_MESSAGE,
    ("255.255.255.255", DISCOVERY_PORT),
)

print("Discovery request sent; waiting for responses...")

try:
    while True:
        data, addr = sock.recvfrom(4096)

        source_ip, source_port = addr

        # Qu responses contain a NUL-terminated device name.
        name = data.rstrip(b"\x00").decode("ascii", errors="replace")

        print()
        print("Found Qu:")
        print(f"  Name:        {name}")
        print(f"  IP:          {source_ip}")
        print(f"  Port:        {source_port}")
        print(f"  Payload:     {data.hex(' ')} ({len(data)} bytes)")

except socket.timeout:
    print("No more responses.")

finally:
    sock.close()

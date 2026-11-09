# Network packet sniffer

A small Linux packet sniffer written in Rust with no packet-capture library. It opens a raw
`AF_PACKET` socket, reads Ethernet frames, parses the IPv4, TCP and UDP headers by hand,
and prints:

- the payload of plaintext **HTTP** traffic (TCP port 80)
- a one-line summary of every **UDP** packet

HTTPS and other encrypted traffic is not decrypted. It only shows that packets exist.

> Only run this on networks you own or have permission to monitor.
> Plaintext HTTP can contain passwords and cookies.

## Run

```bash
cargo build --release
sudo ./target/release/network_packet_sniffer
```

Build as your normal user and run only the binary with `sudo`. Running `sudo cargo run`
leaves root-owned files in `target/`. To avoid `sudo` entirely, grant the binary the raw
socket capability once:

```bash
sudo setcap cap_net_raw=eip target/release/network_packet_sniffer
./target/release/network_packet_sniffer
```

To generate something to capture, open a plain HTTP page in another terminal:

```bash
curl http://neverssl.com
```

## Example output

Replace this with your own capture:

```
Binding to interface: eth0 (Index: 2)
Socket successfully bound to device at Layer 2!
Listening for incoming frames... (Run with sudo)

[UDP] 192.0.2.10:53412 -> 192.0.2.1:53

[TCP] 192.0.2.10:48210 -> 203.0.113.5:80
+++ INSECURE HTTP PAYLOAD CAPTURED +++
GET / HTTP/1.1
Host: neverssl.com
...
++++++++++++++++++++++++++++++++++++++
```

## How it works

1. **Pick an interface.** It reads `/sys/class/net/` and uses the first interface that is
   `up` and is not `lo`.
2. **Open a raw socket.** `AF_PACKET` with `SOCK_RAW` and protocol `ETH_P_ALL` delivers
   whole Ethernet frames in both directions, and it is bound to that interface.
3. **Walk the headers.** Each frame is sliced layer by layer, and every length comes from the
   packet itself:

   | Layer | Size | Used for |
   |-------|------|----------|
   | Ethernet | 14 bytes | EtherType `0x0800` means IPv4 |
   | IPv4 | `IHL * 4` bytes | addresses, protocol (6 = TCP, 17 = UDP) |
   | TCP | data offset `* 4` bytes | ports, then the payload |
   | UDP | 8 bytes | ports |

4. **Print.** Multi-byte fields are big-endian (`u16::from_be_bytes`). TCP payloads on
   port 80 are printed as text.

## Limitations

- IPv4 over Ethernet only. IPv6, ARP and VLAN-tagged frames are ignored.
- The interface is the first one that is up, which may not be your main one if you have
  Docker or VPN interfaces. Tunnel interfaces (such as VPNs) have no Ethernet header, so
  parsing would be wrong on them.
- The receive buffer is 1518 bytes, so larger frames are truncated.
- No TCP stream reassembly or IP fragment handling. Each segment is printed on its own.
- Uses `unsafe` for the socket address and for reading the receive buffer.

## Ideas to extend it

- Take the interface name as an argument.
- Add `#[cfg(test)]` tests for `Ipv4Header::new` using hand-built byte arrays.
- Filter by port or host.
- Parse IPv6.
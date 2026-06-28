use socket2::{Domain, Protocol, Socket, Type};
use std::mem::MaybeUninit;
use std::net::Ipv4Addr;
use std::{fs, io};

// --- IPv4 Header Struct and Impl ---
#[derive(Debug)]
pub struct Ipv4Header {
    pub version: u8,
    pub ihl: u8,
    pub dscp: u8,
    pub ecn: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub header_checksum: u16,
    pub source_ip: Ipv4Addr,
    pub dest_ip: Ipv4Addr,
}

impl Ipv4Header {
    pub fn new(data: &[u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }

        let version = data[0] >> 4;
        let ihl = data[0] & 0x0F;
        let dscp = data[1] >> 2;
        let ecn = data[1] & 0x03;
        let total_length = u16::from_be_bytes([data[2], data[3]]);
        let identification = u16::from_be_bytes([data[4], data[5]]);
        let flags = data[6] >> 5;
        let fragment_offset = u16::from_be_bytes([data[6] & 0x1F, data[7]]);
        let ttl = data[8];
        let protocol = data[9];
        let header_checksum = u16::from_be_bytes([data[10], data[11]]);
        let source_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
        let dest_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);

        Some(Self {
            version,
            ihl,
            dscp,
            ecn,
            total_length,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            header_checksum,
            source_ip,
            dest_ip,
        })
    }
}

fn main() -> io::Result<()> {
    let network_location = "/sys/class/net/";
    let mut active_ifindex: Option<i32> = None;
    let mut active_name: String = String::new();

    // 1. Find the active interface index
    for entry in fs::read_dir(network_location)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let operstate_path = path.join("operstate");
            let ifindex_path = path.join("ifindex");

            if let Ok(operstate) = fs::read_to_string(&operstate_path) {
                if let Ok(ifindex_str) = fs::read_to_string(&ifindex_path) {
                    let name = path.file_name().unwrap().to_string_lossy().into_owned();
                    let state = operstate.trim();

                    if name != "lo" && state == "up" {
                        let index = ifindex_str
                            .trim()
                            .parse::<i32>()
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                        active_ifindex = Some(index);
                        active_name = name;
                        break;
                    }
                }
            }
        }
    }

    let ifindex = match active_ifindex {
        Some(idx) => {
            println!("Binding to interface: {} (Index: {})", active_name, idx);
            idx
        }
        None => {
            eprintln!("Error: No active network interface found (excluding 'lo').");
            std::process::exit(1);
        }
    };

    // 2. Open the Raw Socket
    let domain = Domain::from(libc::AF_PACKET);
    let ty = Type::from(libc::SOCK_RAW);
    let protocol = Protocol::from(0x0003u16.to_be() as i32);
    let socket = Socket::new(domain, ty, Some(protocol))?;

    // 3. Bind the socket to the interface
    let mut storage = unsafe { std::mem::zeroed() };
    unsafe {
        let sll = &mut storage as *mut _ as *mut libc::sockaddr_ll;
        (*sll).sll_family = libc::AF_PACKET as u16;
        (*sll).sll_protocol = 0x0003u16.to_be();
        (*sll).sll_ifindex = ifindex;
    }
    let addr =
        unsafe { socket2::SockAddr::new(storage, std::mem::size_of::<libc::sockaddr_ll>() as u32) };
    socket.bind(&addr)?;
    println!("Socket successfully bound to device at Layer 2!");

    // 4. Safe Memory Buffer
    let mut buffer = [MaybeUninit::<u8>::uninit(); 1518];
    println!("Listening for incoming frames... (Run with sudo)\n");

    loop {
        let (bytes_read, _address) = socket.recv_from(&mut buffer)?;
        let packet =
            unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, bytes_read) };

        if bytes_read < 14 {
            continue;
        }
        let ethertype = u16::from_be_bytes([packet[12], packet[13]]);

        if ethertype == 0x0800 {
            let ip_payload = &packet[14..];

            if let Some(ipv4_header) = Ipv4Header::new(ip_payload) {
                let ipv4_header_len = (ipv4_header.ihl as usize) * 4;

                if ip_payload.len() >= ipv4_header_len {
                    let transport_data = &ip_payload[ipv4_header_len..];

                    // TCP Parsing (Protocol 6)
                    if ipv4_header.protocol == 6 && transport_data.len() >= 20 {
                        let src_port = u16::from_be_bytes([transport_data[0], transport_data[1]]);
                        let dest_port = u16::from_be_bytes([transport_data[2], transport_data[3]]);

                        // TCP Data Offset is stored in the top 4 bits of Byte 12
                        let tcp_data_offset = (transport_data[12] >> 4) as usize;
                        let tcp_header_len = tcp_data_offset * 4;

                        // Check if it is unencrypted HTTP traffic (Port 80)
                        if (src_port == 80 || dest_port == 80)
                            && transport_data.len() > tcp_header_len
                        {
                            let payload = &transport_data[tcp_header_len..];

                            if !payload.is_empty() {
                                // Convert raw bytes to text, replacing invalid characters
                                let payload_text = String::from_utf8_lossy(payload);

                                println!(
                                    "\n[TCP] {}:{} -> {}:{}",
                                    ipv4_header.source_ip, src_port, ipv4_header.dest_ip, dest_port
                                );
                                println!("+++ INSECURE HTTP PAYLOAD CAPTURED +++");
                                println!("{}", payload_text.trim());
                                println!("++++++++++++++++++++++++++++++++++++++\n");
                            }
                        }
                    }
                    // UDP Parsing (Protocol 17) just to keep visibility active
                    else if ipv4_header.protocol == 17 && transport_data.len() >= 8 {
                        let src_port = u16::from_be_bytes([transport_data[0], transport_data[1]]);
                        let dest_port = u16::from_be_bytes([transport_data[2], transport_data[3]]);
                        println!(
                            "[UDP] {}:{} -> {}:{}",
                            ipv4_header.source_ip, src_port, ipv4_header.dest_ip, dest_port
                        );
                    }
                }
            }
        }
    }
}

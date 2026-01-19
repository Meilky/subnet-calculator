fn create_netmask(cidr: u8) -> (u32, u32) {
    let right_len = 32 - cidr;
    let netmask = (u32::MAX >> right_len) << right_len;

    (netmask, !netmask)
}

pub struct SubNet {
    pub network: u32,
    pub first: u32,
    pub last: u32,
    pub broadcast: u32,
    pub nb_usable_ip: u32,
    pub cidr: u8,
}

impl SubNet {
    pub fn new(ip: u32, cidr: u8, nb_usable_ip: u32) -> SubNet {
        let network: u32;
        let first: u32;
        let last: u32;
        let broadcast: u32;

        if cidr == 32 {
            network = ip;
            first = ip;
            last = ip;
            broadcast = ip;
        } else if cidr == 31 {
            network = ip & 0xfffffffe;
            first = network;
            last = network + 1;
            broadcast = last;
        } else {
            let (netmask, wildmask) = create_netmask(cidr);
            network = ip & netmask;
            first = network + 1;
            broadcast = network + wildmask;
            last = broadcast - 1;
        }

        SubNet {
            network,
            first,
            last,
            broadcast,
            cidr,
            nb_usable_ip,
        }
    }
}

impl std::fmt::Display for SubNet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let network: [u8; 4] = self.network.to_be_bytes();
        let first: [u8; 4] = self.first.to_be_bytes();
        let last: [u8; 4] = self.last.to_be_bytes();
        let broadcast: [u8; 4] = self.broadcast.to_be_bytes();

        write!(f, "{{\n\t\"network\": \"{}.{}.{}.{}\",\n\t\"first\": \"{}.{}.{}.{}\",\n\t\"last\": \"{}.{}.{}.{}\",\n\t\"broadcast\": \"{}.{}.{}.{}\",\n\t\"nbUsableIp\": {}\n}}", 
            network[0],
            network[1],
            network[2],
            network[3],
            first[0],
            first[1],
            first[2],
            first[3],
            last[0],
            last[1],
            last[2],
            last[3],
            broadcast[0],
            broadcast[1],
            broadcast[2],
            broadcast[3],
            self.nb_usable_ip
        )
    }
}

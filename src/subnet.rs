use crate::utils::{create_broadcast, create_netmask, create_network, Cidr, Ip};

pub struct SubNet {
    pub network: Ip,
    pub first: Ip,
    pub last: Ip,
    pub broadcast: Ip,
    pub cidr: Cidr,
    pub nb_usable_ip: u32,
}

impl SubNet {
    pub fn from(ip: Ip, cidr: Cidr) -> Result<Self, String> {
        let base: u32 = 2;
        let network: Ip;
        let first: Ip;
        let last: Ip;
        let broadcast: Ip;
        let nb_usable_ip: u32;

        if cidr == 32 {
            network = ip;
            first = ip;
            last = ip;
            broadcast = ip;
            nb_usable_ip = 0;
        } else if cidr == 31 {
            network = create_network(&0xfffffffe, &ip);
            first = network;
            last = network + 1;
            broadcast = last;
            nb_usable_ip = 0;
        } else {
            let (netmask, wildmask) = create_netmask(cidr)?;
            network = create_network(&netmask, &ip);
            first = network + 1;
            broadcast = create_broadcast(&network, &wildmask);
            last = broadcast - 1;
            nb_usable_ip = base.pow((32 - cidr).into()) - 2;
        }

        Ok(SubNet {
            network,
            first,
            last,
            broadcast,
            cidr,
            nb_usable_ip,
        })
    }

    pub fn split_in_subs(&self, cidr: Cidr) -> Result<Vec<String>, String> {
        let mut vec: Vec<String> = vec![];

        if cidr < self.cidr {
            return Err("Cidr can't be smaller than the current subnet one!".to_string());
        }

        if cidr == 32 {
            return Ok(vec![self.serialise_json()]);
        }

        let base: u32 = 2;

        let mut next_network = self.network;

        for _i in 0..base.pow(cidr - self.cidr) {
            let next_sub = SubNet::from(next_network, cidr)?;
            next_network = next_sub.broadcast + 1;

            vec.push(next_sub.serialise_json());
        }

        Ok(vec)
    }

    pub fn serialise_json(&self) -> String {
        let network: [u8; 4] = self.network.to_be_bytes();
        let first: [u8; 4] = self.first.to_be_bytes();
        let last: [u8; 4] = self.last.to_be_bytes();
        let broadcast: [u8; 4] = self.broadcast.to_be_bytes();

        format!(
            "{{\n\t\"network\": \"{}.{}.{}.{}\",\n\t\"first\": \"{}.{}.{}.{}\",\n\t\"last\": \"{}.{}.{}.{}\",\n\t\"broadcast\": \"{}.{}.{}.{}\",\n\t\"nbUsableIp\": {}\n}}",
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

impl std::fmt::Display for SubNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialise_json())
    }
}

pub fn parse_ip(raw_ip: String) -> Result<u32, String> {
    let split_ip: Vec<&str> = raw_ip.split(".").collect();

    if split_ip.len() != 4 {
        return Err("The inserted value isn't an proprely formated ip!".to_string());
    }

    let mut ip: u32 = 0;
    let base: u32 = 2;

    for i in 0..4 {
        let s = match split_ip.get(i) {
            Some(v) => v,
            _ => return Err(format!("Invalid ip at position (no value): {}", i)),
        };

        let value = match s.parse::<u32>() {
            Ok(v) => v,
            _ => return Err(format!("Invalid ip at position (no a int): {}", i)),
        };

        ip += value
            * base.pow(
                ((3 - i) * 8)
                    .try_into()
                    .expect("Error will transforming u8 to u32"),
            );
    }

    Ok(ip)
}

pub fn parse_cidr(raw_cidr: String) -> Result<u8, String> {
    let cidr = raw_cidr.trim().parse::<u8>().expect("Cidr isn't a number");

    if cidr > 32 {
        return Err("The cidr should be under 32!".to_string());
    }

    Ok(cidr)
}

pub fn parse_min_nb(raw_nb: String, cidr: u8) -> Result<u32, String> {
    let base: u32 = 2;
    let nb: u32 = raw_nb
        .trim()
        .parse::<u32>()
        .expect("The min nb of ip is not a number");

    if cidr == 32 || cidr == 31 {
        return Ok(0);
    }

    if base.pow(32 - cidr as u32) - 2 < nb {
        return Err(
            "The min nb cannot be bigger then the max number of usable ip of the network"
                .to_string(),
        );
    }

    Ok(nb)
}

pub fn parse_nb_thread(raw_nb_thread: String) -> Result<u8, String> {
    let thread = raw_nb_thread
        .trim()
        .parse::<u8>()
        .expect("Number of thread isn't a number");

    if thread > 32 {
        return Err("The number of thread should be <= 32!".to_string());
    }

    Ok(thread)
}

pub fn parse_input(
    raw_ip: String,
    raw_nb: String,
    raw_nb_thread: String,
) -> Result<(u32, u8, u32, u8), String> {
    let split_ip: Vec<&str> = raw_ip.trim().split("/").collect();

    if split_ip.len() != 2 {
        return Err("The ip should be formated this way : <ip>/<cidr>".to_string());
    }

    let ip = parse_ip(split_ip.get(0).expect("No ip provided").to_string())?;
    let cidr = parse_cidr(split_ip.get(1).expect("No cidr provided").to_string())?;
    let min_nb: u32 = parse_min_nb(raw_nb, cidr)?;
    let nb_thread = parse_nb_thread(raw_nb_thread)?;

    Ok((ip, cidr, min_nb, nb_thread))
}

const FLASH: usize = 0x8000; // ATmega328p

pub fn load_hex(source: &str) -> Vec<u8> {
    let mut prog: Vec<u8> = vec![0; FLASH];
    let mut n_bytes = 0;
    for line in source.split("\n") {
        if !line.is_empty() && &line[..1] == ":" && &line[7..9] == "00" {
            let bytes = u8::from_str_radix(&line[1..3], 16).unwrap(); // number of bytes of instructions on this line
            let addr = u16::from_str_radix(&line[3..7], 16).unwrap();
            for i in 0..bytes {
                let offset = 9 + (i as usize * 2);
                prog[addr as usize + i as usize] =
                    u8::from_str_radix(&line[offset..offset + 2], 16).unwrap();
            }

            n_bytes += bytes as usize;
        }
    }
    println!("number of instructions: {}", n_bytes / 2);

    prog
}

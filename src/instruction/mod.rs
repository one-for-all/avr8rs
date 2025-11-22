use crate::{
    atmega328p::ATMega328P,
    instruction::instructions::{decode, is_two_word_instruction},
    ternary,
};

pub mod instructions;

pub fn avr_instruction(atmega328p: &mut ATMega328P) {
    let opcode = atmega328p.cpu.prog_mem[atmega328p.cpu.pc as usize];
    let instruction = decode(opcode);

    // println!(
    //     "ins: {:?}, opcode: {:04b} {:04b} {:04b} {:04b}",
    //     instruction,
    //     opcode >> 12 & 0b1111,
    //     opcode >> 8 & 0b1111,
    //     opcode >> 4 & 0b1111,
    //     opcode & 0b1111
    // );

    match instruction {
        instructions::Instruction::ADC => {
            // ADC, 0001 11rd dddd rrrr
            let d = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4);
            let r = atmega328p
                .cpu
                .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5));
            let sum = d as u16 + r as u16 + (atmega328p.cpu.data[95] as u16 & 1);
            let R = (sum & 255) as u8;
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xc0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!((R ^ r) & (d ^ R) & 128, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= ternary!(sum & 256, 1, 0);
            sreg |= ternary!(1 & ((d & r) | (r & !R) | (!R & d)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::ADD => {
            // ADD, 0000 11rd dddd rrrr
            let d = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) as i32;
            let r = atmega328p
                .cpu
                .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5)) as i32;
            let R = (d + r) & 255;
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R as u8);
            let mut sreg = atmega328p.cpu.data[95] & 0xc0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!((R ^ r) & (R ^ d) & 128, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= ternary!((d as u16 + r as u16) & 256, 1, 0);
            sreg |= ternary!(1 & ((d & r) | (r & !R) | (!R & d)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::ADIW => {
            /* ADIW, 1001 0110 KKdd KKKK */
            let addr = 2 * ((opcode & 0x30) >> 4) + 24;
            let value = atmega328p.cpu.get_data_u16(addr);
            let R = (value.wrapping_add((opcode & 0xf) | ((opcode & 0xc0) >> 2))) & 0xffff;
            atmega328p.cpu.set_data_u16(addr, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xe0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(0x8000 & R, 4, 0);
            sreg |= ternary!(!value & R & 0x8000, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= ternary!(!R & value & 0x8000, 1, 0);
            atmega328p.cpu.data[95] = sreg;
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::AND => {
            /* AND, 0010 00rd dddd rrrr */
            let R = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4)
                & atmega328p
                    .cpu
                    .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5));
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xe1;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::ANDI => {
            /* ANDI, 0111 KKKK dddd KKKK */
            let R = atmega328p.cpu.get_data(((opcode & 0xf0) >> 4) + 16)
                & ((opcode & 0xf) | ((opcode & 0xf00) >> 4)) as u8;
            atmega328p.cpu.set_data(((opcode & 0xf0) >> 4) + 16, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xe1;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::ASR => {
            /* ASR, 1001 010d dddd 0101 */
            let value = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4);
            let R = (value >> 1) | (128 & value);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xe0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= value & 1;
            sreg |= ternary!(((sreg >> 2) & 1) ^ (sreg & 1), 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::BCLR => {
            atmega328p.cpu.data[95] &= !(1 << ((opcode & 0x70) >> 4));
        }
        instructions::Instruction::BLD => {
            /* BLD, 1111 100d dddd 0bbb */
            let b = opcode & 7;
            let d = (opcode & 0x1f0) >> 4;
            atmega328p.cpu.set_data(
                d,
                (!(1 << b) & atmega328p.cpu.get_data(d))
                    | (((atmega328p.cpu.data[95] >> 6) & 1) << b),
            );
        }
        instructions::Instruction::BRBC => {
            /* BRBC, 1111 01kk kkkk ksss */
            if (atmega328p.cpu.data[95] & (1 << (opcode & 7))) == 0 {
                atmega328p.cpu.pc = (atmega328p.cpu.pc + ((opcode & 0x1f8) >> 3) as u32)
                    - ternary!(opcode & 0x200, 0x40, 0);
                atmega328p.cpu.cycles += 1;
            }
        }
        instructions::Instruction::BRBS => {
            /* BRBS, 1111 00kk kkkk ksss */
            if atmega328p.cpu.data[95] & (1 << (opcode & 7)) != 0 {
                atmega328p.cpu.pc = atmega328p.cpu.pc + ((opcode & 0x1f8) >> 3) as u32
                    - ternary!(opcode & 0x200, 0x40, 0);
                atmega328p.cpu.cycles += 1;
            }
        }
        instructions::Instruction::BSET => {
            /* BSET, 1001 0100 0sss 1000 */
            atmega328p.cpu.data[95] |= 1 << ((opcode & 0x70) >> 4);
        }
        instructions::Instruction::BST => {
            /* BST, 1111 101d dddd 0bbb */
            let d = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4);
            let b = opcode & 7;
            atmega328p.cpu.data[95] =
                (atmega328p.cpu.data[95] & 0xbf) | (ternary!((d >> b) & 1, 0x40, 0));
        }
        instructions::Instruction::CALL => {
            /* CALL, 1001 010k kkkk 111k kkkk kkkk kkkk kkkk */
            let k = atmega328p.cpu.prog_mem[(atmega328p.cpu.pc + 1) as usize] as u32
                | (((opcode & 1) as u32) << 16)
                | (((opcode & 0x1f0) as u32) << 13);
            let ret = atmega328p.cpu.pc + 2;
            let sp = atmega328p.cpu.get_data_u16(93);
            let pc_22_bits = atmega328p.cpu.pc_22_bits;
            atmega328p.cpu.set_data(sp, 255 & ret as u8);
            atmega328p.cpu.set_data(sp - 1, (ret >> 8) as u8 & 255);
            if pc_22_bits {
                atmega328p.cpu.set_data(sp - 2, (ret >> 16) as u8 & 255);
            }
            atmega328p
                .cpu
                .set_data_u16(93, sp - (if pc_22_bits { 3 } else { 2 }));
            atmega328p.cpu.pc = k - 1;
            atmega328p.cpu.cycles += if pc_22_bits { 4 } else { 3 };
        }
        instructions::Instruction::CBI => {
            /* CBI, 1001 1000 AAAA Abbb */
            let A = opcode & 0xf8;
            let b = opcode & 7;
            let R = atmega328p.cpu.read_data((A >> 3) + 32);
            let mask = 1 << b;
            atmega328p.write_data_with_mask((A >> 3) + 32, R & !mask, mask);
        }
        instructions::Instruction::COM => {
            /* COM, 1001 010d dddd 0000 */
            let d = (opcode & 0x1f0) >> 4;
            let R = 255 - atmega328p.cpu.get_data(d);
            atmega328p.cpu.set_data(d, R);
            let mut sreg = (atmega328p.cpu.data[95] & 0xe1) | 1;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::CP => {
            /* CP, 0001 01rd dddd rrrr */
            let val1 = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) as i32;
            let val2 = atmega328p
                .cpu
                .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5)) as i32;
            let R = val1 - val2;
            let mut sreg = atmega328p.cpu.data[95] & 0xc0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= if 0 != ((val1 ^ val2) & (val1 ^ R) & 128) {
                8
            } else {
                0
            };
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= if val2 > val1 { 1 } else { 0 };
            sreg |= ternary!(1 & ((!val1 & val2) | (val2 & R) | (R & !val1)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::CPC => {
            /* CPC, 0000 01rd dddd rrrr */
            let arg1 = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) as i32;
            let arg2 = atmega328p
                .cpu
                .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5)) as i32;
            let mut sreg = atmega328p.cpu.data[95];
            let r = arg1 - arg2 - (sreg as i32 & 1);

            // IMPORTANT: check r == 0, not !r != 0
            sreg = (sreg & 0xc0)
                | (if r == 0 && (sreg >> 1) & 1 != 0 { 2 } else { 0 })
                | (if arg2 + (sreg as i32 & 1) > arg1 {
                    1
                } else {
                    0
                });
            sreg |= ternary!(128 & r, 4, 0);
            sreg |= ternary!((arg1 ^ arg2) & (arg1 ^ r) & 128, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= ternary!(1 & ((!arg1 & arg2) | (arg2 & r) | (r & !arg1)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::CPSE => {
            /* CPSE, 0001 00rd dddd rrrr */
            if atmega328p.cpu.get_data((opcode & 0x1f0) >> 4)
                == atmega328p
                    .cpu
                    .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5))
            {
                let next_opcode = atmega328p.cpu.prog_mem[(atmega328p.cpu.pc + 1) as usize];
                let skip_size = if is_two_word_instruction(next_opcode) {
                    2
                } else {
                    1
                };
                atmega328p.cpu.pc += skip_size;
                atmega328p.cpu.cycles += skip_size;
            }
        }
        instructions::Instruction::CPI => {
            /* CPI, 0011 KKKK dddd KKKK */
            let arg1 = atmega328p.cpu.get_data(((opcode & 0xf0) >> 4) + 16) as i32;
            let arg2 = ((opcode & 0xf) | ((opcode & 0xf00) >> 4)) as i32;
            let r = arg1 - arg2;
            let mut sreg = atmega328p.cpu.data[95] & 0xc0;
            sreg |= ternary!(r, 0, 2);
            sreg |= ternary!(128 & r, 4, 0);
            sreg |= ternary!((arg1 ^ arg2) & (arg1 ^ r) & 128, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= if arg2 > arg1 { 1 } else { 0 };
            sreg |= ternary!(1 & ((!arg1 & arg2) | (arg2 & r) | (r & !arg1)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::DEC => {
            /* DEC, 1001 010d dddd 1010 */
            let value = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) as i32;
            let R = value - 1;
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R as u8);
            let mut sreg = atmega328p.cpu.data[95] & 0xe1;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= if 128 == value { 8 } else { 0 };
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::ELPM_INC => {
            /* ELPM(INC), 1001 000d dddd 0111 */
            let rampz = atmega328p.cpu.data[0x5b] as u32;
            let i = atmega328p.cpu.get_data_u16(30);
            atmega328p.cpu.set_data(
                (opcode & 0x1f0) >> 4,
                atmega328p.cpu.prog_bytes[((rampz << 16) | (i as u32)) as usize],
            );
            atmega328p.cpu.set_data_u16(30, i + 1);
            if i == 0xffff {
                atmega328p.cpu.data[0x5b] =
                    ((rampz + 1) % ((atmega328p.cpu.prog_bytes.len() >> 16) as u32)) as u8;
            }
            atmega328p.cpu.cycles += 2;
        }
        instructions::Instruction::EOR => {
            /* EOR, 0010 01rd dddd rrrr */
            let R = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4)
                ^ atmega328p
                    .cpu
                    .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5));
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xe1;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::ICALL => {
            /* ICALL, 1001 0101 0000 1001 */
            let ret_addr = atmega328p.cpu.pc + 1;
            let sp = atmega328p.cpu.get_data_u16(93);
            let pc_22_bits = atmega328p.cpu.pc_22_bits;
            atmega328p.cpu.set_data(sp, (ret_addr & 255) as u8);
            atmega328p
                .cpu
                .set_data(sp - 1, ((ret_addr >> 8) & 255) as u8);
            if pc_22_bits {
                atmega328p
                    .cpu
                    .set_data(sp - 2, ((ret_addr >> 16) & 255) as u8);
            }
            atmega328p
                .cpu
                .set_data_u16(93, sp - if pc_22_bits { 3 } else { 2 });
            atmega328p.cpu.pc = atmega328p.cpu.get_data_u16(30) as u32 - 1;
            atmega328p.cpu.cycles += if pc_22_bits { 3 } else { 2 };
        }
        instructions::Instruction::IJMP => {
            /* IJMP, 1001 0100 0000 1001 */
            atmega328p.cpu.pc = atmega328p.cpu.get_data_u16(30) as u32 - 1;
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::IN => {
            /* IN, 1011 0AAd dddd AAAA */
            let i = atmega328p
                .cpu
                .read_data(((opcode & 0xf) | ((opcode & 0x600) >> 5)) + 32);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, i);
        }
        instructions::Instruction::INC => {
            /* INC, 1001 010d dddd 0011 */
            let d = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4);
            let r = (d.wrapping_add(1)) & 255;
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, r);
            let mut sreg = atmega328p.cpu.data[95] & 0xe1;
            sreg |= ternary!(r, 0, 2);
            sreg |= ternary!(128 & r, 4, 0);
            sreg |= if 127 == d { 8 } else { 0 };
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::JMP => {
            /* JMP, 1001 010k kkkk 110k kkkk kkkk kkkk kkkk */
            atmega328p.cpu.pc = (atmega328p.cpu.prog_mem[(atmega328p.cpu.pc + 1) as usize] as u32
                | ((opcode as u32 & 1) << 16)
                | ((opcode as u32 & 0x1f0) << 13))
                - 1;
            atmega328p.cpu.cycles += 2;
        }
        instructions::Instruction::LDI => {
            /* LDI, 1110 KKKK dddd KKKK */
            // println!(
            //     "LDI: {:08b}",
            //     (opcode & 0xf) as u8 | ((opcode & 0xf00) >> 4) as u8
            // );
            atmega328p.cpu.set_data(
                ((opcode & 0xf0) >> 4) + 16,
                (opcode & 0xf) as u8 | ((opcode & 0xf00) >> 4) as u8,
            );
        }
        instructions::Instruction::LDS => {
            /* LDS, 1001 000d dddd 0000 kkkk kkkk kkkk kkkk */
            atmega328p.cpu.cycles += 1;
            let value = atmega328p
                .cpu
                .read_data(atmega328p.cpu.prog_mem[(atmega328p.cpu.pc + 1) as usize]);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, value);
            atmega328p.cpu.pc += 1;
        }
        instructions::Instruction::LDX => {
            /* LDX, 1001 000d dddd 1100 */
            atmega328p.cpu.cycles += 1;
            let data = atmega328p.cpu.read_data(atmega328p.cpu.get_data_u16(26));
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, data);
        }
        instructions::Instruction::LDX_INC => {
            /* LDX(INC), 1001 000d dddd 1101 */
            let x = atmega328p.cpu.get_data_u16(26);
            atmega328p.cpu.cycles += 1;
            let data = atmega328p.cpu.read_data(x);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, data);
            atmega328p.cpu.set_data_u16(26, x + 1);
        }
        instructions::Instruction::LDY => {
            /* LDY, 1000 000d dddd 1000 */
            atmega328p.cpu.cycles += 1;
            let data = atmega328p.cpu.read_data(atmega328p.cpu.get_data_u16(28));
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, data);
        }
        instructions::Instruction::LDY_INC => {
            /* LDY(INC), 1001 000d dddd 1001 */
            let y = atmega328p.cpu.get_data_u16(28);
            atmega328p.cpu.cycles += 1;
            let data = atmega328p.cpu.read_data(y);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, data);
            atmega328p.cpu.set_data_u16(28, y + 1);
        }
        instructions::Instruction::LDDY => {
            /* LDDY, 10q0 qq0d dddd 1qqq */
            atmega328p.cpu.cycles += 1;
            let addr = atmega328p.cpu.get_data_u16(28)
                + ((opcode & 7) | ((opcode & 0xc00) >> 7) | ((opcode & 0x2000) >> 8));
            let data = atmega328p.cpu.read_data(addr);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, data);
        }
        instructions::Instruction::LDZ => {
            /* LDZ, 1000 000d dddd 0000 */
            atmega328p.cpu.cycles += 1;
            let data = atmega328p.cpu.read_data(atmega328p.cpu.get_data_u16(30));
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, data);
        }
        instructions::Instruction::LDZ_INC => {
            /* LDZ(INC), 1001 000d dddd 0001 */
            let z = atmega328p.cpu.get_data_u16(30);
            atmega328p.cpu.cycles += 1;
            let data = atmega328p.cpu.read_data(z);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, data);
            atmega328p.cpu.set_data_u16(30, z + 1);
        }
        instructions::Instruction::LDDZ => {
            /* LDDZ, 10q0 qq0d dddd 0qqq */
            atmega328p.cpu.cycles += 1;
            let addr = atmega328p.cpu.get_data_u16(30)
                + ((opcode & 7) | ((opcode & 0xc00) >> 7) | ((opcode & 0x2000) >> 8));
            let data = atmega328p.cpu.read_data(addr);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, data);
        }
        instructions::Instruction::LPM_REG => {
            /* LPM(REG), 1001 000d dddd 0100 */
            atmega328p.cpu.set_data(
                (opcode & 0x1f0) >> 4,
                atmega328p.cpu.prog_bytes[atmega328p.cpu.get_data_u16(30) as usize],
            );
            atmega328p.cpu.cycles += 2;
        }
        instructions::Instruction::LPM_INC => {
            /* LPM(INC), 1001 000d dddd 0101 */
            let i = atmega328p.cpu.get_data_u16(30);
            atmega328p
                .cpu
                .set_data((opcode & 0x1f0) >> 4, atmega328p.cpu.prog_bytes[i as usize]);
            atmega328p.cpu.set_data_u16(30, i + 1);
            atmega328p.cpu.cycles += 2;
        }
        instructions::Instruction::LSR => {
            /* LSR, 1001 010d dddd 0110 */
            let value = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4);
            let R = value >> 1;
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xe0;
            sreg |= ternary!(R, 0, 2);
            sreg |= value & 1;
            sreg |= ternary!(((sreg >> 2) & 1) ^ (sreg & 1), 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::MOV => {
            /* MOV, 0010 11rd dddd rrrr */
            atmega328p.cpu.set_data(
                (opcode & 0x1f0) >> 4,
                atmega328p
                    .cpu
                    .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5)),
            );
        }
        instructions::Instruction::MOVW => {
            /* MOVW, 0000 0001 dddd rrrr */
            let r2 = 2 * (opcode & 0xf);
            let d2 = 2 * ((opcode & 0xf0) >> 4);
            atmega328p.cpu.set_data(d2, atmega328p.cpu.get_data(r2));
            atmega328p
                .cpu
                .set_data(d2 + 1, atmega328p.cpu.get_data(r2 + 1));
        }
        instructions::Instruction::MUL => {
            /* MUL, 1001 11rd dddd rrrr */
            let R = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) as u16
                * atmega328p
                    .cpu
                    .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5)) as u16;
            atmega328p.cpu.set_data_u16(0, R);
            atmega328p.cpu.data[95] = (atmega328p.cpu.data[95] & 0xfc)
                | (ternary!(0xffff & R, 0, 2))
                | (ternary!(0x8000 & R, 1, 0));
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::NEG => {
            /* NEG, 1001 010d dddd 0001 */
            let d = (opcode & 0x1f0) >> 4;
            let value = atmega328p.cpu.get_data(d) as i32;
            let R = 0 - value;
            atmega328p.cpu.set_data(d, R as u8);
            let mut sreg = atmega328p.cpu.data[95] & 0xc0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= if 128 == R { 8 } else { 0 };
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= ternary!(R, 1, 0);
            sreg |= ternary!(1 & (R | value), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::OR => {
            /* OR, 0010 10rd dddd rrrr */
            let R = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4)
                | atmega328p
                    .cpu
                    .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5));
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xe1;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::OUT => {
            atmega328p.write_data_with_mask(
                ((opcode & 0xf) | ((opcode & 0x600) >> 5)) + 32,
                atmega328p.cpu.get_data((opcode & 0x1f0) >> 4),
                0xff,
            );
        }
        instructions::Instruction::POP => {
            /* POP, 1001 000d dddd 1111 */
            let value = atmega328p.cpu.get_data_u16(93) + 1;
            atmega328p.cpu.set_data_u16(93, value);
            atmega328p
                .cpu
                .set_data((opcode & 0x1f0) >> 4, atmega328p.cpu.get_data(value));
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::PUSH => {
            /* PUSH, 1001 001d dddd 1111 */
            let value = atmega328p.cpu.get_data_u16(93);
            atmega328p
                .cpu
                .set_data(value, atmega328p.cpu.get_data((opcode & 0x1f0) >> 4));
            atmega328p.cpu.set_data_u16(93, value - 1);
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::RCALL => {
            /* RCALL, 1101 kkkk kkkk kkkk */
            let k = (opcode & 0x7ff) as i32 - ternary!(opcode & 0x800, 0x800, 0) as i32;
            let ret_addr = atmega328p.cpu.pc + 1;
            let sp = atmega328p.cpu.get_data_u16(93);
            let pc_22_bits = atmega328p.cpu.pc_22_bits;
            atmega328p.cpu.set_data(sp, (255 & ret_addr) as u8);
            atmega328p
                .cpu
                .set_data(sp - 1, ((ret_addr >> 8) & 255) as u8);
            if pc_22_bits {
                atmega328p
                    .cpu
                    .set_data(sp - 2, ((ret_addr >> 16) & 255) as u8);
            }
            atmega328p
                .cpu
                .set_data_u16(93, sp - (if pc_22_bits { 3 } else { 2 }));
            atmega328p.cpu.pc = atmega328p.cpu.pc.wrapping_add(k as u32);
            // atmega328p.cpu.pc = (atmega328p.cpu.pc as i64 + k as i64) as u32;
            atmega328p.cpu.cycles += if pc_22_bits { 3 } else { 2 };
        }
        instructions::Instruction::RET => {
            /* RET, 1001 0101 0000 1000 */
            let pc_22_bits = atmega328p.cpu.pc_22_bits;
            let i = atmega328p.cpu.get_data_u16(93) + if pc_22_bits { 3 } else { 2 };
            atmega328p.cpu.set_data_u16(93, i);
            atmega328p.cpu.pc = ((atmega328p.cpu.get_data(i - 1) as u32) << 8)
                + atmega328p.cpu.get_data(i) as u32
                - 1;
            if pc_22_bits {
                atmega328p.cpu.pc |= (atmega328p.cpu.get_data(i - 2) as u32) << 16;
            }
            atmega328p.cpu.cycles += if pc_22_bits { 4 } else { 3 };
        }
        instructions::Instruction::RETI => {
            /* RETI, 1001 0101 0001 1000 */
            let pc_22_bits = atmega328p.cpu.pc_22_bits;
            let i = atmega328p.cpu.get_data_u16(93) + (if pc_22_bits { 3 } else { 2 });
            atmega328p.cpu.set_data_u16(93, i);
            atmega328p.cpu.pc = ((atmega328p.cpu.get_data(i - 1) as u32) << 8)
                + atmega328p.cpu.get_data(i) as u32
                - 1;
            if pc_22_bits {
                atmega328p.cpu.pc |= (atmega328p.cpu.get_data(i - 2) as u32) << 16;
            }
            atmega328p.cpu.cycles += if pc_22_bits { 4 } else { 3 };
            atmega328p.cpu.data[95] |= 0x80; // Enable interrupts
        }
        instructions::Instruction::RJMP => {
            /* RJMP, 1100 kkkk kkkk kkkk */
            atmega328p.cpu.pc =
                atmega328p.cpu.pc + (opcode & 0x7ff) as u32 - ternary!(opcode & 0x800, 0x800, 0);
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::ROR => {
            /* ROR, 1001 010d dddd 0111 */

            let d = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4);
            let r = (d >> 1) | ((atmega328p.cpu.data[95] & 1) << 7);
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, r);
            let mut sreg = atmega328p.cpu.data[95] & 0xe0;
            sreg |= ternary!(r, 0, 2);
            sreg |= ternary!(128 & r, 4, 0);
            sreg |= ternary!(1 & d, 1, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ (sreg & 1), 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::SBC => {
            /* SBC, 0000 10rd dddd rrrr */
            let val1 = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) as i32;
            let val2 = atmega328p
                .cpu
                .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5)) as i32;
            let mut sreg = atmega328p.cpu.data[95];
            // let R = val1.wrapping_sub(val2).wrapping_sub(sreg & 1);
            let R = val1 - val2 - (sreg & 1) as i32;
            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R as u8);
            sreg = (sreg & 0xc0)
                | (if R == 0 && (sreg >> 1) & 1 != 0 { 2 } else { 0 })
                | (if val2 as u16 + (sreg as u16 & 1) > val1 as u16 {
                    1
                } else {
                    0
                });
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!((val1 ^ val2) & (val1 ^ R) & 128, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= ternary!(1 & ((!val1 & val2) | (val2 & R) | (R & !val1)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::SBCI => {
            /* SBCI, 0100 KKKK dddd KKKK */
            let val1 = atmega328p.cpu.get_data(((opcode & 0xf0) >> 4) + 16) as i32;
            let val2 = ((opcode & 0xf) | ((opcode & 0xf00) >> 4)) as i32;
            let mut sreg = atmega328p.cpu.data[95];
            // let R = val1.wrapping_sub(val2).wrapping_sub(sreg & 1);
            let R = val1 - val2 - (sreg & 1) as i32;
            atmega328p
                .cpu
                .set_data(((opcode & 0xf0) >> 4) + 16, R as u8);
            sreg = (sreg & 0xc0)
                | (if R == 0 && (sreg >> 1) & 1 != 0 { 2 } else { 0 })
                | (if val2 as u16 + (sreg as u16 & 1) > val1 as u16 {
                    1
                } else {
                    0
                });
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!((val1 ^ val2) & (val1 ^ R) & 128, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= ternary!(1 & ((!val1 & val2) | (val2 & R) | (R & !val1)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::SBI => {
            /* SBI, 1001 1010 AAAA Abbb */
            let target = ((opcode & 0xf8) >> 3) + 32;
            let mask = 1 << (opcode & 7);
            let data = atmega328p.cpu.read_data(target) | mask;
            atmega328p.write_data_with_mask(target, data, mask);
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::SBIS => {
            /* SBIS, 1001 1011 AAAA Abbb */
            let value = atmega328p.cpu.read_data(((opcode & 0xf8) >> 3) + 32);
            if value & (1 << (opcode & 7)) != 0 {
                let next_opcode = atmega328p.cpu.prog_mem[(atmega328p.cpu.pc + 1) as usize];
                let skip_size = if is_two_word_instruction(next_opcode) {
                    2
                } else {
                    1
                };
                atmega328p.cpu.cycles += skip_size;
                atmega328p.cpu.pc += skip_size;
            }
        }
        instructions::Instruction::SBIW => {
            /* SBIW, 1001 0111 KKdd KKKK */
            let i = 2 * ((opcode & 0x30) >> 4) + 24;
            let a = atmega328p.cpu.get_data_u16(i) as i32;
            let l = ((opcode & 0xf) | ((opcode & 0xc0) >> 2)) as i32;
            let R = a - l;
            atmega328p.cpu.set_data_u16(i, R as u16);
            let mut sreg = atmega328p.cpu.data[95] & 0xc0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(0x8000 & R, 4, 0);
            sreg |= ternary!(a & !R & 0x8000, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= if l > a { 1 } else { 0 };
            sreg |= ternary!(1 & ((!a & l) | (l & R) | (R & !a)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::SBR => {
            /* SBR, 0110 KKKK dddd KKKK */
            let R = atmega328p.cpu.get_data(((opcode & 0xf0) >> 4) + 16)
                | ((opcode & 0xf) as u8 | ((opcode & 0xf00) >> 4) as u8);
            atmega328p.cpu.set_data(((opcode & 0xf0) >> 4) + 16, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xe1;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::SBRC => {
            /* SBRC, 1111 110r rrrr 0bbb */
            if atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) & (1 << (opcode & 7)) == 0 {
                let next_opcode = atmega328p.cpu.prog_mem[(atmega328p.cpu.pc + 1) as usize];
                let skip_size = if is_two_word_instruction(next_opcode) {
                    2
                } else {
                    1
                };
                atmega328p.cpu.cycles += skip_size;
                atmega328p.cpu.pc += skip_size;
            }
        }
        instructions::Instruction::SBRS => {
            /* SBRS, 1111 111r rrrr 0bbb */
            if atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) & (1 << (opcode & 7)) != 0 {
                let next_opcode = atmega328p.cpu.prog_mem[(atmega328p.cpu.pc + 1) as usize];
                let skip_size = if is_two_word_instruction(next_opcode) {
                    2
                } else {
                    1
                };
                atmega328p.cpu.cycles += skip_size;
                atmega328p.cpu.pc += skip_size;
            }
        }
        instructions::Instruction::STS => {
            /* STS, 1001 001d dddd 0000 kkkk kkkk kkkk kkkk */
            let value = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4);
            let addr = atmega328p.cpu.prog_mem[(atmega328p.cpu.pc + 1) as usize];
            atmega328p.write_data_with_mask(addr, value, 0xff);
            atmega328p.cpu.pc += 1;
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::STX => {
            /* STX, 1001 001r rrrr 1100 */
            atmega328p.write_data_with_mask(
                atmega328p.cpu.get_data_u16(26),
                atmega328p.cpu.get_data((opcode & 0x1f0) >> 4),
                0xff,
            );
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::STX_INC => {
            /* STX(INC), 1001 001r rrrr 1101 */
            let x = atmega328p.cpu.get_data_u16(26);
            atmega328p.write_data_with_mask(
                x,
                atmega328p.cpu.get_data((opcode & 0x1f0) >> 4),
                0xff,
            );
            atmega328p.cpu.set_data_u16(26, x + 1);
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::STX_DEC => {
            /* STX(DEC), 1001 001r rrrr 1110 */
            let i = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4);
            let x = atmega328p.cpu.get_data_u16(26) - 1;
            atmega328p.cpu.set_data_u16(26, x);
            atmega328p.write_data_with_mask(x, i, 0xff);
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::STDY => {
            /* STDY, 10q0 qq1r rrrr 1qqq */
            atmega328p.write_data_with_mask(
                atmega328p.cpu.get_data_u16(28)
                    + ((opcode & 7) | ((opcode & 0xc00) >> 7) | ((opcode & 0x2000) >> 8)),
                atmega328p.cpu.get_data((opcode & 0x1f0) >> 4),
                0xff,
            );
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::STZ => {
            /* STZ, 1000 001r rrrr 0000 */
            atmega328p.write_data_with_mask(
                atmega328p.cpu.get_data_u16(30),
                atmega328p.cpu.get_data((opcode & 0x1f0) >> 4),
                0xff,
            );
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::STDZ => {
            /* STDZ, 10q0 qq1r rrrr 0qqq */
            atmega328p.write_data_with_mask(
                atmega328p.cpu.get_data_u16(30)
                    + ((opcode & 7) | ((opcode & 0xc00) >> 7) | ((opcode & 0x2000) >> 8)),
                atmega328p.cpu.get_data((opcode & 0x1f0) >> 4),
                0xff,
            );
            atmega328p.cpu.cycles += 1;
        }
        instructions::Instruction::SUB => {
            /* SUB, 0001 10rd dddd rrrr */
            let val1 = atmega328p.cpu.get_data((opcode & 0x1f0) >> 4) as i32;
            let val2 = atmega328p
                .cpu
                .get_data((opcode & 0xf) | ((opcode & 0x200) >> 5)) as i32;
            let R = val1 - val2;

            atmega328p.cpu.set_data((opcode & 0x1f0) >> 4, R as u8);
            let mut sreg = atmega328p.cpu.data[95] & 0xc0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!((val1 ^ val2) & (val1 ^ R) & 128, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= if val2 > val1 { 1 } else { 0 };
            sreg |= ternary!(1 & ((!val1 & val2) | (val2 & R) | (R & !val1)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
        instructions::Instruction::SUBI => {
            /* SUBI, 0101 KKKK dddd KKKK */
            let val1 = atmega328p.cpu.get_data(((opcode & 0xf0) >> 4) + 16) as u8;
            let val2 = ((opcode & 0xf) | ((opcode & 0xf00) >> 4)) as u8;
            let R = val1.wrapping_sub(val2);
            atmega328p.cpu.set_data(((opcode & 0xf0) >> 4) + 16, R);
            let mut sreg = atmega328p.cpu.data[95] & 0xc0;
            sreg |= ternary!(R, 0, 2);
            sreg |= ternary!(128 & R, 4, 0);
            sreg |= ternary!((val1 ^ val2) & (val1 ^ R) & 128, 8, 0);
            sreg |= ternary!(((sreg >> 2) & 1) ^ ((sreg >> 3) & 1), 0x10, 0);
            sreg |= if val2 > val1 { 1 } else { 0 };
            sreg |= ternary!(1 & ((!val1 & val2) | (val2 & R) | (R & !val1)), 0x20, 0);
            atmega328p.cpu.data[95] = sreg;
        }
    }

    atmega328p.cpu.pc = (atmega328p.cpu.pc + 1) % atmega328p.cpu.prog_mem.len() as u32;
    atmega328p.cpu.cycles += 1;
}

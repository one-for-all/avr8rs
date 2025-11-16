#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    ADC,
    ADD,
    ADIW,
    AND,
    ANDI,
    ASR,
    BCLR,
    BLD,
    BRBC,
    BRBS,
    BSET,
    BST,
    CALL,
    CBI,
    COM,
    CP,
    CPC,
    CPSE,
    CPI,
    DEC,
    ELPM_INC,
    EOR,
    ICALL,
    IJMP,
    IN,
    INC,
    JMP,
    LDI,
    LDS,
    LDX,
    LDX_INC,
    LDY,
    LDDY,
    LDZ,
    LDZ_INC,
    LDDZ,
    LPM_REG,
    LPM_INC,
    LSR,
    MOV,
    MOVW,
    MUL,
    NEG,
    OR,
    OUT,
    POP,
    PUSH,
    RCALL,
    RET,
    RETI,
    RJMP,
    ROR,
    SBC,
    SBCI,
    SBI,
    SBIS,
    SBIW,
    SBR,
    SBRC,
    SBRS,
    STS,
    STX,
    STX_INC,
    STDY,
    SUB,
    SUBI,
}

pub fn decode(opcode: u16) -> Instruction {
    if opcode & 0xfc00 == 0x1c00 {
        /* ADC, 0001 11rd dddd rrrr */
        Instruction::ADC
    } else if opcode & 0xfc00 == 0xc00 {
        /* ADD, 0000 11rd dddd rrrr */
        Instruction::ADD
    } else if opcode & 0xff00 == 0x9600 {
        /* ADIW, 1001 0110 KKdd KKKK */
        Instruction::ADIW
    } else if opcode & 0xfc00 == 0x2000 {
        Instruction::AND
    } else if opcode & 0xf000 == 0x7000 {
        Instruction::ANDI
    } else if opcode & 0xfe0f == 0x9405 {
        Instruction::ASR
    } else if opcode & 0xff8f == 0x9488 {
        Instruction::BCLR
    } else if opcode & 0xfe08 == 0xf800 {
        Instruction::BLD
    } else if opcode & 0xfc00 == 0xf400 {
        Instruction::BRBC
    } else if opcode & 0xfc00 == 0xf000 {
        Instruction::BRBS
    } else if opcode & 0xff8f == 0x9408 {
        Instruction::BSET
    } else if opcode & 0xfe08 == 0xfa00 {
        Instruction::BST
    } else if opcode & 0xfe0e == 0x940e {
        Instruction::CALL
    } else if opcode & 0xff00 == 0x9800 {
        Instruction::CBI
    } else if opcode & 0xfe0f == 0x9400 {
        Instruction::COM
    } else if opcode & 0xfc00 == 0x1400 {
        Instruction::CP
    } else if opcode & 0xfc00 == 0x400 {
        Instruction::CPC
    } else if opcode & 0xfc00 == 0x1000 {
        Instruction::CPSE
    } else if opcode & 0xf000 == 0x3000 {
        Instruction::CPI
    } else if opcode & 0xfe0f == 0x940a {
        Instruction::DEC
    } else if opcode & 0xfe0f == 0x9007 {
        Instruction::ELPM_INC
    } else if opcode & 0xfc00 == 0x2400 {
        Instruction::EOR
    } else if opcode == 0x9509 {
        Instruction::ICALL
    } else if opcode == 0x9409 {
        Instruction::IJMP
    } else if opcode & 0xf800 == 0xb000 {
        Instruction::IN
    } else if opcode & 0xfe0f == 0x9403 {
        Instruction::INC
    } else if opcode & 0xfe0e == 0x940c {
        Instruction::JMP
    } else if opcode & 0xf000 == 0xe000 {
        Instruction::LDI
    } else if opcode & 0xfe0f == 0x9000 {
        Instruction::LDS
    } else if opcode & 0xfe0f == 0x900c {
        Instruction::LDX
    } else if opcode & 0xfe0f == 0x900d {
        Instruction::LDX_INC
    } else if opcode & 0xfe0f == 0x8008 {
        Instruction::LDY
    } else if opcode & 0xd208 == 0x8008
        && (opcode & 7) | ((opcode & 0xc00) >> 7) | ((opcode & 0x2000) >> 8) != 0
    {
        Instruction::LDDY
    } else if opcode & 0xfe0f == 0x8000 {
        Instruction::LDZ
    } else if opcode & 0xfe0f == 0x9001 {
        Instruction::LDZ_INC
    } else if opcode & 0xd208 == 0x8000
        && (opcode & 7) | ((opcode & 0xc00) >> 7) | ((opcode & 0x2000) >> 8) != 0
    {
        Instruction::LDDZ
    } else if opcode & 0xfe0f == 0x9004 {
        Instruction::LPM_REG
    } else if opcode & 0xfe0f == 0x9005 {
        Instruction::LPM_INC
    } else if opcode & 0xfe0f == 0x9406 {
        Instruction::LSR
    } else if opcode & 0xfc00 == 0x2c00 {
        Instruction::MOV
    } else if opcode & 0xff00 == 0x100 {
        Instruction::MOVW
    } else if opcode & 0xfc00 == 0x9c00 {
        Instruction::MUL
    } else if opcode & 0xfe0f == 0x9401 {
        Instruction::NEG
    } else if opcode & 0xfc00 == 0x2800 {
        Instruction::OR
    } else if opcode & 0xf800 == 0xb800 {
        Instruction::OUT
    } else if opcode & 0xfe0f == 0x900f {
        Instruction::POP
    } else if opcode & 0xfe0f == 0x920f {
        Instruction::PUSH
    } else if opcode & 0xf000 == 0xd000 {
        Instruction::RCALL
    } else if opcode == 0x9508 {
        Instruction::RET
    } else if opcode == 0x9518 {
        Instruction::RETI
    } else if opcode & 0xf000 == 0xc000 {
        Instruction::RJMP
    } else if opcode & 0xfe0f == 0x9407 {
        Instruction::ROR
    } else if opcode & 0xfc00 == 0x800 {
        Instruction::SBC
    } else if opcode & 0xf000 == 0x4000 {
        Instruction::SBCI
    } else if opcode & 0xff00 == 0x9a00 {
        Instruction::SBI
    } else if opcode & 0xff00 == 0x9b00 {
        Instruction::SBIS
    } else if opcode & 0xff00 == 0x9700 {
        Instruction::SBIW
    } else if opcode & 0xf000 == 0x6000 {
        Instruction::SBR
    } else if opcode & 0xfe08 == 0xfc00 {
        Instruction::SBRC
    } else if opcode & 0xfe08 == 0xfe00 {
        Instruction::SBRS
    } else if opcode & 0xfe0f == 0x9200 {
        Instruction::STS
    } else if opcode & 0xfe0f == 0x920c {
        Instruction::STX
    } else if opcode & 0xfe0f == 0x920d {
        Instruction::STX_INC
    } else if opcode & 0xd208 == 0x8208
        && (opcode & 7) | ((opcode & 0xc00) >> 7) | ((opcode & 0x2000) >> 8) != 0
    {
        Instruction::STDY
    } else if opcode & 0xfc00 == 0x1800 {
        Instruction::SUB
    } else if opcode & 0xf000 == 0x5000 {
        Instruction::SUBI
    } else {
        panic!("instruction not implemented: {:#018b}", opcode)
    }
}

pub fn is_two_word_instruction(opcode: u16) -> bool {
    /* LDS */
    (opcode & 0xfe0f) == 0x9000 ||
    /* STS */
    (opcode & 0xfe0f) == 0x9200 ||
    /* CALL */
    (opcode & 0xfe0e) == 0x940e ||
    /* JMP */
    (opcode & 0xfe0e) == 0x940c
}

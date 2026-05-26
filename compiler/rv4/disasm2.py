import struct

with open('examples/fact.o','rb') as f:
    d = f.read()

def sext(x, bits):
    if x & (1 << (bits-1)):
        return x - (1 << bits)
    return x

# Section 4 (.text.fact)
sh_offset=0x5a
sh_size=0x66
data = d[sh_offset:sh_offset+sh_size]

print('=== .text.fact (fact + putdec) ===')
print('Section: offset=0x{:x}, size=0x{:x}'.format(sh_offset, sh_size))

def disasm16(inst):
    op = inst & 0x3
    funct3 = (inst >> 13) & 0x7
    rd = (inst >> 7) & 0x1f
    rs2 = (inst >> 2) & 0x1f
    rdp = ((inst >> 2) & 0x7) + 8
    rs2p = ((inst >> 2) & 0x7) + 8
    rs1p = ((inst >> 7) & 0x7) + 8

    if op == 0 and funct3 == 0:
        imm = (((inst >> 7) & 1) << 3) | ((inst >> 1) & 4) | ((inst >> 4) & 0x38) | ((inst >> 2) & 0x40)
        if imm == 0: return 'C.NOP (reserved)'
        return 'C.ADDI4SPN rd=x{} imm={}'.format(rdp, imm)
    elif op == 0 and funct3 == 2:
        imm = (((inst >> 5) & 0x38) | ((inst >> 1) & 4) | ((inst >> 7) & 4))
        return 'C.LW rd=x{} offset={}(x{})'.format(rdp, imm, rs1p)
    elif op == 0 and funct3 == 6:
        imm = (((inst >> 5) & 0x38) | ((inst >> 1) & 4) | ((inst >> 7) & 4))
        return 'C.SW rs2=x{} offset={}(x{})'.format(rs2p, imm, rs1p)
    elif op == 1 and funct3 == 0:
        imm = sext(((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f), 6)
        return 'C.ADDI rd=x{} imm={}'.format(rd, imm)
    elif op == 1 and funct3 == 1 and rd == 0:
        return 'C.NOP'
    elif op == 1 and funct3 == 1:
        imm = sext(((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f), 6)
        return 'C.ADDIW rd=x{} imm={}'.format(rd, imm)
    elif op == 1 and funct3 == 2:
        imm = sext(((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f), 6)
        return 'C.LI rd=x{} imm={}'.format(rd, imm)
    elif op == 1 and funct3 == 3:
        if rd == 2:
            imm = sext(((inst >> 6) & 0x20) | ((inst >> 3) & 0x10) | ((inst >> 5) & 8) | ((inst << 1) & 0x100) | ((inst >> 2) & 0x40) | ((inst >> 1) & 0x180), 9)
            return 'C.ADDI16SP imm={}'.format(imm)
        else:
            imm = ((inst >> 12) & 1) << 5 | ((inst >> 2) & 0x1f)
            return 'C.LUI rd=x{} imm=0x{:x}'.format(rd, imm)
    elif op == 1 and funct3 == 4:
        fa = (inst >> 12) & 3
        sa = (inst >> 5) & 3
        if fa==0 and sa==0: return 'C.SRLI x{}'.format(rs1p)
        elif fa==0 and sa==1: return 'C.SRAI x{}'.format(rs1p)
        elif fa==1 and sa==0: return 'C.ANDI x{}'.format(rs1p)
        elif fa==2 and sa==0: return 'C.SUB x{}'.format(rs1p)
        elif fa==2 and sa==1: return 'C.XOR x{}'.format(rs1p)
        elif fa==2 and sa==2: return 'C.OR x{}'.format(rs1p)
        elif fa==2 and sa==3: return 'C.AND x{}'.format(rs1p)
        elif fa==3 and sa==0: return 'C.SUBW x{}'.format(rs1p)
        elif fa==3 and sa==1: return 'C.ADDW x{}'.format(rs1p)
        else: return 'C.RESERVED'
    elif op == 1 and funct3 == 5: return 'C.J'
    elif op == 1 and funct3 == 6: return 'C.BEQZ'
    elif op == 1 and funct3 == 7: return 'C.BNEZ'
    elif op == 2 and funct3 == 0:
        sh = ((inst >> 7) & 0x3c) | ((inst >> 2) & 3)
        return 'C.SLLI rd=x{} sh={}'.format(rd, sh)
    elif op == 2 and funct3 == 2: return 'C.LWSP rd=x{}'.format(rd)
    elif op == 2 and funct3 == 4:
        r1 = (inst >> 7) & 0x1f
        r2 = (inst >> 2) & 0x1f
        if r2 == 0:
            return 'C.JR x{}'.format(r1) if r1 != 0 else 'C.EBREAK'
        else: return 'C.MV x{}=x{}'.format(r1, r2)
    elif op == 2 and funct3 == 6: return 'C.SWSP rs2=x{}'.format(rs2)
    elif op == 2 and funct3 == 7: return 'C.SDSP rs2=x{}'.format(rs2)
    elif op == 0 and funct3 == 3: return 'C.LD (RV64)'
    elif op == 0 and funct3 == 7: return 'C.SD (RV64)'
    elif op == 2 and funct3 == 3: return 'C.LDSP (RV64)'
    else: return 'UNKNOWN op={} funct3={}'.format(op, funct3)

offset = 0
while offset < sh_size:
    b0 = data[offset]
    if (b0 & 0x3) != 0x3:
        # 16-bit
        inst = data[offset] | (data[offset+1] << 8)
        name = disasm16(inst)
        print('  {:#06x}: {:#06x}  {}'.format(offset + 0x10000, inst, name))
        offset += 2
    else:
        # 32-bit
        inst = (data[offset] | (data[offset+1] << 8) | 
                (data[offset+2] << 16) | (data[offset+3] << 24))
        opcode = inst & 0x7f
        rd_v = (inst >> 7) & 0x1f
        funct3_v = (inst >> 12) & 0x7
        rs1_v = (inst >> 15) & 0x1f
        rs2_v = (inst >> 20) & 0x1f
        funct7_v = (inst >> 25) & 0x7f
        
        if opcode == 0x37:
            name = 'LUI rd=x{} imm=0x{:x}'.format(rd_v, (inst >> 12) & 0xfffff)
        elif opcode == 0x17:
            name = 'AUIPC rd=x{} imm=0x{:x}'.format(rd_v, (inst >> 12) & 0xfffff)
        elif opcode == 0x6f:
            imm_v = ((inst >> 31) & 1) << 20 | ((inst >> 21) & 0x3ff) << 1 | ((inst >> 20) & 1) << 11 | ((inst >> 12) & 0xff) << 12
            imm_v = sext(imm_v, 21)
            name = 'JAL rd=x{} imm={}'.format(rd_v, imm_v)
        elif opcode == 0x67:
            imm_v = sext(inst >> 20, 12)
            name = 'JALR rd=x{} rs1=x{} imm={}'.format(rd_v, rs1_v, imm_v)
        elif opcode == 0x63:
            imm_b12 = (inst >> 31) & 1
            imm_b10_5 = (inst >> 25) & 0x3f
            imm_b4_1 = (inst >> 8) & 0xf
            imm_b11 = (inst >> 7) & 1
            imm_v = imm_b12 << 12 | imm_b11 << 11 | imm_b10_5 << 5 | imm_b4_1 << 1
            imm_v = sext(imm_v, 13)
            names = {0:'BEQ',1:'BNE',4:'BLT',5:'BGE',6:'BLTU',7:'BGEU'}
            name = '{} x{},x{} imm={}'.format(names.get(funct3_v, "B???"), rs1_v, rs2_v, imm_v)
        elif opcode == 0x03:
            names = {0:'LB',1:'LH',2:'LW',3:'LD',4:'LBU',5:'LHU',6:'LWU'}
            imm_v = sext(inst >> 20, 12)
            name = '{} x{},{}(x{})'.format(names.get(funct3_v, "L???"), rd_v, imm_v, rs1_v)
        elif opcode == 0x23:
            names = {0:'SB',1:'SH',2:'SW',3:'SD'}
            imm_v = ((inst >> 25) << 5) | ((inst >> 7) & 0x1f)
            imm_v = sext(imm_v, 12)
            name = '{} x{},{}(x{})'.format(names.get(funct3_v, "S???"), rs2_v, imm_v, rs1_v)
        elif opcode == 0x13:
            imm_v = sext(inst >> 20, 12)
            if funct3_v == 5:
                n2 = 'SRAI' if inst & 0x40000000 else 'SRLI'
                name = '{} x{},x{},{}'.format(n2, rd_v, rs1_v, imm_v & 0x3f)
            else:
                names2 = {0:'ADDI',1:'SLLI',2:'SLTI',3:'SLTIU',4:'XORI',6:'ORI',7:'ANDI'}
                name = '{} x{},x{},{}'.format(names2.get(funct3_v, "OP-IMM???"), rd_v, rs1_v, imm_v)
        elif opcode == 0x33:
            n = {0:'ADD',1:'SLL',2:'SLT',3:'SLTU',4:'XOR',5:'SRL',6:'OR',7:'AND'}.get(funct3_v, 'OP???')
            if funct7_v & 0x20:
                if funct3_v == 0: n = 'SUB'
                elif funct3_v == 5: n = 'SRA'
            elif funct7_v & 1:
                mulnames = {0:'MUL',1:'MULH',2:'MULHSU',3:'MULHU',4:'DIV',5:'DIVU',6:'REM',7:'REMU'}
                n = mulnames.get(funct3_v, 'MUL???')
            name = '{} x{},x{},x{}'.format(n, rd_v, rs1_v, rs2_v)
        elif opcode == 0x73:
            if funct3_v == 0:
                bits = (inst >> 20) & 0xfff
                if bits == 0: name = 'ECALL'
                elif bits == 1: name = 'EBREAK'
                else: name = 'CSRRx bits={:#04x}'.format(bits)
            else: name = 'SYSTEM funct3={}'.format(funct3_v)
        else: name = 'UNKNOWN opcode={:#04x}'.format(opcode)
        print('  {:#06x}: {:#010x}  {}'.format(offset + 0x10000, inst, name))
        offset += 4
    if offset >= sh_size:
        break

"""Read ordinary PE DLL imports without executing an untrusted binary."""
import struct

def imports(data):
    def u16(offset): return struct.unpack_from('<H', data, offset)[0]
    def u32(offset): return struct.unpack_from('<I', data, offset)[0]
    assert data[:2] == b'MZ'
    pe = u32(0x3c)
    assert data[pe:pe+4] == b'PE\0\0' and u16(pe+4) == 0x8664
    count, size = u16(pe+6), u16(pe+20)
    optional = pe+24
    assert u16(optional) == 0x20b and count < 128
    sections = optional+size
    def offset(rva):
        for i in range(count):
            section = sections+i*40
            start, length, file_offset = u32(section+12), u32(section+16), u32(section+20)
            if start <= rva < start+length:
                result = file_offset+rva-start
                assert result < len(data)
                return result
        raise ValueError('PE RVA outside file-backed sections')
    rva = u32(optional+112+8)
    if not rva: return []
    descriptor = offset(rva)
    result = []
    for i in range(512):
        entry = descriptor+i*20
        if data[entry:entry+20] == bytes(20): return result
        name = offset(u32(entry+12))
        end = data.index(b'\0', name, name+256)
        result.append(data[name:end].decode('ascii').lower())
    raise ValueError('Too many PE import descriptors')

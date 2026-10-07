"""Public synthetic assets for memory qualification; no private source data."""
import struct
import zlib

def public_image(path):
    def chunk(kind, data):
        return struct.pack('>I', len(data))+kind+data+struct.pack('>I', zlib.crc32(kind+data))
    rows=b''.join(b'\0'+b''.join(bytes((20,90,220)) if 16<=x<48 and 16<=y<48 else bytes((255,255,255)) for x in range(64)) for y in range(64))
    path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',64,64,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b''))

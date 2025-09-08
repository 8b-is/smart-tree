#!/usr/bin/env python3
import base64
import zlib
import json

def decode_m8_capsule(data):
    """Decode an M8 capsule from Omni"""
    lines = data.strip().split('\n')
    
    # Parse header
    header = lines[0].split(',')
    magic = header[0]
    compressed = header[1] == 'True'
    
    # Get the base64 data
    b64_data = lines[1]
    
    print(f"MEM|8 Capsule Decoder")
    print(f"Magic: {magic}")
    print(f"Compressed: {compressed}")
    print(f"Data length: {len(b64_data)} chars")
    print("-" * 50)
    
    # Decode base64
    decoded = base64.b64decode(b64_data)
    print(f"Decoded size: {len(decoded)} bytes")
    
    # Decompress if needed
    if compressed:
        try:
            decompressed = zlib.decompress(decoded)
            print(f"Decompressed size: {len(decompressed)} bytes")
            
            # Try to interpret as JSON
            try:
                data = json.loads(decompressed)
                print("\nDecoded content (JSON):")
                print(json.dumps(data, indent=2))
            except:
                # Try as plain text
                try:
                    text = decompressed.decode('utf-8')
                    print("\nDecoded content (Text):")
                    print(text)
                except:
                    # Show hex dump of first 256 bytes
                    print("\nDecoded content (Hex - first 256 bytes):")
                    hex_view = ' '.join(f'{b:02x}' for b in decompressed[:256])
                    print(hex_view)
                    
                    # Try to find readable strings
                    print("\nReadable strings found:")
                    current_string = ""
                    for byte in decompressed:
                        if 32 <= byte <= 126:  # Printable ASCII
                            current_string += chr(byte)
                        else:
                            if len(current_string) > 4:
                                print(f"  '{current_string}'")
                            current_string = ""
                    
        except Exception as e:
            print(f"Decompression error: {e}")
            print("\nRaw decoded bytes (first 256):")
            print(' '.join(f'{b:02x}' for b in decoded[:256]))
    
# The M8 capsule data
capsule_data = """MEM8,True,eJydk8Fy0zAQhu95isXnxHGSBlxfGMr02OGQDgcKk1HtjS2IJY+0Jsl0wrOzUmTHCeUAF421+rX77f7yywgg+onGSq2iDKKH+4d0/XkWjV08NygICxefJ/PlJEknyfIxSbPZ2yxZxrezJH2XfDlpRYmKnHJVC0OPBvEUt2i73Fhr4k+xXeO+2Woj3C5U0opw7+6/8JYDjaDK3fk1bYz+jjnZqXWJJ9RlZlGBDVXrWhd4kb47PtezxH3YPrvDMvnUXfraFukCeS3mt3C/z6Xrd9ypFKd2+dst2rO84BUXN3CnzVAcl5KuRDcpfFB7qVsbedUxkNm25mYOQ6AN4laqcgjpWghEGczH5+izL5zBYhAToU4GsxA89mS2LUu0FGz4qJWVBRqwP2TTcE0YtgmKfQCSNcaBeRRSRTUP1EM/+YMenQ6NN6CQho1ykr5yZ6Mfdx8NxnjTrkdeoM2NbLrzT7oaw64SjMSP0baGCcVOSHr/5mKk/wJz6eprVM+Xzl4xrRAVSCbSGmqhDn5aNoZVmGccx//NdnpErzF1Bv+V6mAJa7DUbjYx3CERW6w0AbNZ2EmqmHng6TfvKQn/5J7Cu/zjF+sAwnYjeWa+UC8Y/Mou6+j4G+d6M78="""

decode_m8_capsule(capsule_data)
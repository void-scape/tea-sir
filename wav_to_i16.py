import wave
import struct
import sys

def wav_to_i16_array(filename):
    with wave.open(filename, 'rb') as wav:
        frames = wav.readframes(-1)
        samples = struct.unpack('<' + 'h' * (len(frames) // 2), frames)
    
    output_file = filename.rsplit('.', 1)[0] + '.bin'
    with open(output_file, 'wb') as f:
        for sample in samples:
            le_bytes = sample.to_bytes(2, byteorder='little', signed=True)
            f.write(le_bytes)
    print(f"Generated {output_file}")

wav_to_i16_array(sys.argv[1])

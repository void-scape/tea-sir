from PIL import Image
import struct
import sys
import os

def image_to_srgba_array(filename):
    try:
        with Image.open(filename) as img:
            rgba_img = img.convert('RGBA')
            width, height = rgba_img.size
            pixel_data = rgba_img.tobytes()
        
        output_file = filename.rsplit('.', 1)[0] + '.bin'
        
        with open(output_file, 'wb') as f:
            f.write(struct.pack('<I', width))
            f.write(struct.pack('<I', height))
            f.write(pixel_data)
        
        print(f"Generated {output_file}")
        print(f"Image dimensions: {width}x{height}")
        print(f"Total file size: {8 + len(pixel_data)} bytes")
        return True
    except Exception as e:
        print(f"Error processing {filename}: {e}")
        return False

def process_directory(directory):
    # Common image extensions
    image_extensions = {'.jpg', '.jpeg', '.png', '.gif', '.bmp', '.tiff', '.tga', '.webp'}
    
    processed_count = 0
    error_count = 0
    
    for filename in os.listdir(directory):
        file_path = os.path.join(directory, filename)
        
        # Skip if not a file
        if not os.path.isfile(file_path):
            continue
            
        # Check if it has an image extension
        _, ext = os.path.splitext(filename.lower())
        if ext in image_extensions:
            print(f"\nProcessing: {filename}")
            if image_to_srgba_array(file_path):
                processed_count += 1
            else:
                error_count += 1
    
    print(f"\n=== Summary ===")
    print(f"Successfully processed: {processed_count} images")
    if error_count > 0:
        print(f"Errors: {error_count} files")

if len(sys.argv) != 2:
    print("Usage: python script.py <directory>")
    sys.exit(1)

directory = sys.argv[1]

if not os.path.isdir(directory):
    print(f"Error: '{directory}' is not a valid directory")
    sys.exit(1)

process_directory(directory)

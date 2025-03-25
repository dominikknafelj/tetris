from PIL import Image, ImageDraw
import os

def create_tetris_icon(size):
    # Create a new image with a black background
    img = Image.new('RGB', (size, size), 'black')
    draw = ImageDraw.Draw(img)
    
    # Calculate block size (divide icon into 4x4 grid)
    block_size = size // 4
    padding = block_size // 8  # Small padding between blocks
    
    # Colors for Tetris pieces
    colors = [
        (0, 240, 240),  # Cyan (I piece)
        (240, 160, 0),  # Orange (L piece)
        (0, 0, 240),    # Blue (J piece)
        (240, 240, 0),  # Yellow (O piece)
    ]
    
    # Draw an arrangement of Tetris blocks that form a "T" shape
    blocks = [
        # Top bar of T (I piece)
        [(0, 0), (1, 0), (2, 0), (3, 0)],
        # Stem of T (combination of pieces)
        [(1, 1), (1, 2), (1, 3)],
        # Base decoration
        [(0, 3), (2, 3)]
    ]
    
    # Draw each set of blocks with its color
    for color_idx, block_set in enumerate(blocks):
        color = colors[color_idx % len(colors)]
        for x, y in block_set:
            # Calculate block position with padding
            left = x * block_size + padding
            top = y * block_size + padding
            right = (x + 1) * block_size - padding
            bottom = (y + 1) * block_size - padding
            
            # Draw block with a slight 3D effect
            # Main block
            draw.rectangle([left, top, right, bottom], fill=color)
            
            # Highlight (top and left edges)
            highlight_color = tuple(min(c + 40, 255) for c in color)
            draw.line([left, top, right, top], fill=highlight_color, width=2)
            draw.line([left, top, left, bottom], fill=highlight_color, width=2)
            
            # Shadow (bottom and right edges)
            shadow_color = tuple(max(c - 40, 0) for c in color)
            draw.line([left, bottom, right, bottom], fill=shadow_color, width=2)
            draw.line([right, top, right, bottom], fill=shadow_color, width=2)
    
    return img

def generate_mac_icons():
    """Generate icons in all sizes needed for macOS"""
    # Create icons directory if it doesn't exist
    if not os.path.exists('icons'):
        os.makedirs('icons')
    
    # Required sizes for macOS icons
    sizes = [16, 32, 64, 128, 256, 512, 1024]
    
    for size in sizes:
        icon = create_tetris_icon(size)
        icon.save(f'icons/icon_{size}x{size}.png')
        print(f"Generated {size}x{size} icon")

if __name__ == "__main__":
    generate_mac_icons() 
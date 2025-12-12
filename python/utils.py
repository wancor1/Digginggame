import math
from constants import FONT_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, BLOCK_SIZE

def estimate_text_width(text):
    estimated_width = 0
    for char in text:
        if char == '\n':
            continue
        code_point = ord(char)
        if code_point < 128 or (0xFF61 <= code_point <= 0xFF9F):
            estimated_width += FONT_SIZE / 2
        else:
            estimated_width += FONT_SIZE
    return estimated_width

def calculate_text_center_position(box_width, box_height, text_content):
    if not text_content:
        return box_width / 2, (box_height - FONT_SIZE) / 2

    text_width_pixels = estimate_text_width(text_content)
    text_y = (box_height - FONT_SIZE) / 2
    text_x = (box_width - text_width_pixels) / 2
    return text_x, text_y

def world_to_chunk_coords(world_x, world_y):
    chunk_x = math.floor(world_x / (CHUNK_SIZE_X_BLOCKS * BLOCK_SIZE))
    chunk_y = math.floor(world_y / (CHUNK_SIZE_Y_BLOCKS * BLOCK_SIZE))
    return chunk_x, chunk_y

def world_to_relative_in_chunk_coords(world_x, world_y):
    cx, cy = world_to_chunk_coords(world_x, world_y)
    rel_bx = (world_x // BLOCK_SIZE) - cx * CHUNK_SIZE_X_BLOCKS
    rel_by = (world_y // BLOCK_SIZE) - cy * CHUNK_SIZE_Y_BLOCKS
    return rel_bx, rel_by

def chunk_coords_to_world_origin(chunk_x, chunk_y):
    world_x = chunk_x * CHUNK_SIZE_X_BLOCKS * BLOCK_SIZE
    world_y = chunk_y * CHUNK_SIZE_Y_BLOCKS * BLOCK_SIZE
    return world_x, world_y

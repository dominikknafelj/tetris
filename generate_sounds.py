import wave
import struct
import math
import os

def generate_square_wave(frequency, duration, amplitude=0.5, sample_rate=32768):
    """Generate a square wave with the given frequency and duration."""
    num_samples = int(duration * sample_rate)
    samples = []
    for i in range(num_samples):
        t = i / sample_rate
        # Square wave is 1 for half the period, -1 for the other half
        sample = amplitude if math.sin(2 * math.pi * frequency * t) >= 0 else -amplitude
        samples.append(sample)
    return samples

def save_wave_file(filename, samples, sample_rate=32768):
    """Save samples as a WAV file."""
    with wave.open(filename, 'w') as wave_file:
        wave_file.setnchannels(1)  # Mono
        wave_file.setsampwidth(2)  # 2 bytes per sample
        wave_file.setframerate(sample_rate)
        
        # Convert samples to bytes
        sample_data = []
        for sample in samples:
            # Convert to 16-bit integer
            sample = int(sample * 32767)
            sample_data.append(struct.pack('h', sample))
            
        wave_file.writeframes(b''.join(sample_data))

def create_move_sound():
    """Create a short descending tone for piece movement."""
    # GameBoy-style descending tone
    samples = []
    for freq in [440, 392, 349]:  # A4, G4, F4
        samples.extend(generate_square_wave(freq, 0.05, 0.3))
    save_wave_file('assets/sounds/move.wav', samples)

def create_rotate_sound():
    """Create a quick ascending arpeggio for piece rotation."""
    # GameBoy-style ascending arpeggio
    samples = []
    for freq in [440, 554, 659, 880]:  # A4, C#5, E5, A5
        samples.extend(generate_square_wave(freq, 0.03, 0.3))
    save_wave_file('assets/sounds/rotate.wav', samples)

def create_drop_sound():
    """Create a low impact sound for piece dropping."""
    # GameBoy-style impact sound
    samples = []
    # Start with a high frequency that quickly drops
    duration = 0.1
    sample_rate = 32768
    num_samples = int(duration * sample_rate)
    for i in range(num_samples):
        t = i / sample_rate
        freq = 880 * (1 - t/duration)  # Sweep from 880Hz to 0Hz
        sample = 0.4 if math.sin(2 * math.pi * freq * t) >= 0 else -0.4
        # Add decay
        sample *= 1 - (i / num_samples)
        samples.append(sample)
    save_wave_file('assets/sounds/drop.wav', samples)

def create_clear_sound():
    """Create an upward sweep sound for line clearing."""
    # GameBoy-style clear sound
    samples = []
    duration = 0.2
    sample_rate = 32768
    num_samples = int(duration * sample_rate)
    for i in range(num_samples):
        t = i / sample_rate
        freq = 440 + (880 * t / duration)  # Sweep from 440Hz to 1320Hz
        sample = 0.4 if math.sin(2 * math.pi * freq * t) >= 0 else -0.4
        samples.append(sample)
    save_wave_file('assets/sounds/clear.wav', samples)

def create_game_over_sound():
    """Create a descending sequence for game over."""
    # GameBoy-style game over sound
    samples = []
    freqs = [880, 659, 554, 440, 330]  # A5, E5, C#5, A4, E4
    for freq in freqs:
        samples.extend(generate_square_wave(freq, 0.1, 0.4))
    save_wave_file('assets/sounds/game_over.wav', samples)

def main():
    """Create all sound effects."""
    # Ensure the sounds directory exists
    os.makedirs('assets/sounds', exist_ok=True)
    
    # Generate all sound effects
    create_move_sound()
    create_rotate_sound()
    create_drop_sound()
    create_clear_sound()
    create_game_over_sound()
    
    print("Sound effects generated successfully!")

if __name__ == '__main__':
    main() 
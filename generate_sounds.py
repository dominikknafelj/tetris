import wave
import struct
import math
import os
import numpy as np

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
    return samples

def create_rotate_sound():
    """Create a quick ascending arpeggio for piece rotation."""
    # GameBoy-style ascending arpeggio
    samples = []
    for freq in [440, 554, 659, 880]:  # A4, C#5, E5, A5
        samples.extend(generate_square_wave(freq, 0.03, 0.3))
    return samples

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
    return samples

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
    return samples

def create_game_over_sound():
    """Create a descending sequence for game over."""
    # GameBoy-style game over sound
    samples = []
    freqs = [880, 659, 554, 440, 330]  # A5, E5, C#5, A4, E4
    for freq in freqs:
        samples.extend(generate_square_wave(freq, 0.1, 0.4))
    return samples

def create_tetris_sound():
    """Creates a special sound for clearing 4 rows at once (Tetris)"""
    samples = []
    # Base frequencies for a C major chord progression (C -> F -> G -> C)
    base_freqs = [
        [523.25, 659.25, 783.99],  # C major (C5, E5, G5)
        [698.46, 880.00, 1046.50], # F major (F5, A5, C6)
        [783.99, 987.77, 1174.66], # G major (G5, B5, D6)
        [1046.50, 1318.51, 1567.98] # C major (C6, E6, G6)
    ]
    
    # Duration for each note (in seconds)
    durations = [0.06, 0.06, 0.06, 0.12]  # Shorter notes for arpeggio, longer for final chord
    volumes = [0.3, 0.3, 0.3, 0.4]  # Lower volumes to prevent clipping
    
    # Create an arpeggio effect with the chord progression
    for chord_idx, chord in enumerate(base_freqs):
        # For the final chord, play all notes together
        if chord_idx == len(base_freqs) - 1:
            chord_samples = None
            for freq in chord:
                wave = generate_square_wave(freq, durations[chord_idx], volumes[chord_idx] / len(chord))
                if chord_samples is None:
                    chord_samples = wave
                else:
                    chord_samples = [sum(x) for x in zip(chord_samples, wave)]
            samples.extend(chord_samples)
        else:
            # For other chords, play notes in sequence for arpeggio effect
            for freq in chord:
                samples.extend(generate_square_wave(freq, durations[chord_idx], volumes[chord_idx]))
    
    # Normalize samples to prevent clipping
    max_amplitude = max(abs(min(samples)), abs(max(samples)))
    if max_amplitude > 0:
        scale_factor = 0.9 / max_amplitude  # Leave some headroom
        samples = [s * scale_factor for s in samples]
    
    return samples

def main():
    """Create all sound effects."""
    # Ensure the sounds directory exists
    os.makedirs('assets/sounds', exist_ok=True)
    
    # Generate all sound effects
    save_wave_file('assets/sounds/move.wav', create_move_sound())
    save_wave_file('assets/sounds/rotate.wav', create_rotate_sound())
    save_wave_file('assets/sounds/drop.wav', create_drop_sound())
    save_wave_file('assets/sounds/clear.wav', create_clear_sound())
    save_wave_file('assets/sounds/game_over.wav', create_game_over_sound())
    save_wave_file('assets/sounds/tetris.wav', create_tetris_sound())
    
    print("Sound effects generated successfully!")

if __name__ == '__main__':
    main() 
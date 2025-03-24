import wave
import struct
import math
import os
import numpy as np

def generate_square_wave(frequency, duration, amplitude=0.5, sample_rate=44100):
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

def create_background_music():
    """Creates the Yorcksche Marsch background music directly from the score"""
    sample_rate = 44100
    tempo = 192  # Quarter notes per minute
    quarter_duration = 60.0 / tempo  # Duration of a quarter note in seconds
    
    # Define note frequencies (A4 = 440Hz as reference)
    def note_freq(semitones_from_a4):
        return 440 * (2 ** (semitones_from_a4 / 12.0))
    
    # Full melody (Piano RH) - First section
    melody = [
        # First measure (after the rest)
        (note_freq(2), quarter_duration * 0.75),  # B4 dotted eighth
        (note_freq(4), quarter_duration * 0.25),  # C#5 sixteenth
        (note_freq(5), quarter_duration * 0.5),   # D5 eighth
        (note_freq(7), quarter_duration * 0.5),   # E5 eighth
        (note_freq(9), quarter_duration * 0.5),   # F#5 eighth
        (note_freq(7), quarter_duration * 0.5),   # E5 eighth
        
        # Second measure
        (note_freq(5), quarter_duration * 0.75),  # D5 dotted eighth
        (note_freq(7), quarter_duration * 0.25),  # E5 sixteenth
        (note_freq(9), quarter_duration * 0.5),   # F#5 eighth
        (note_freq(11), quarter_duration * 0.5),  # G#5 eighth
        (note_freq(12), quarter_duration * 0.5),  # A5 eighth
        (note_freq(11), quarter_duration * 0.5),  # G#5 eighth
        
        # Third measure
        (note_freq(9), quarter_duration * 0.75),  # F#5 dotted eighth
        (note_freq(7), quarter_duration * 0.25),  # E5 sixteenth
        (note_freq(5), quarter_duration * 0.5),   # D5 eighth
        (note_freq(4), quarter_duration * 0.5),   # C#5 eighth
        (note_freq(2), quarter_duration * 0.5),   # B4 eighth
        (note_freq(0), quarter_duration * 0.5),   # A4 eighth

        # Fourth measure (repeat of first)
        (note_freq(2), quarter_duration * 0.75),  # B4 dotted eighth
        (note_freq(4), quarter_duration * 0.25),  # C#5 sixteenth
        (note_freq(5), quarter_duration * 0.5),   # D5 eighth
        (note_freq(7), quarter_duration * 0.5),   # E5 eighth
        (note_freq(9), quarter_duration * 0.5),   # F#5 eighth
        (note_freq(7), quarter_duration * 0.5),   # E5 eighth

        # Fifth measure (similar to second)
        (note_freq(5), quarter_duration * 0.75),  # D5 dotted eighth
        (note_freq(7), quarter_duration * 0.25),  # E5 sixteenth
        (note_freq(9), quarter_duration * 0.5),   # F#5 eighth
        (note_freq(11), quarter_duration * 0.5),  # G#5 eighth
        (note_freq(12), quarter_duration * 0.5),  # A5 eighth
        (note_freq(11), quarter_duration * 0.5),  # G#5 eighth

        # Sixth measure
        (note_freq(9), quarter_duration),         # F#5 quarter
        (note_freq(14), quarter_duration),        # B5 quarter
        (note_freq(14), quarter_duration),        # B5 quarter

        # Seventh measure
        (note_freq(14), quarter_duration),        # B5 quarter
        (note_freq(12), quarter_duration * 0.5),  # A5 eighth
        (note_freq(11), quarter_duration * 0.5),  # G#5 eighth
        (note_freq(9), quarter_duration),         # F#5 quarter

        # Eighth measure
        (note_freq(7), quarter_duration),         # E5 quarter
        (note_freq(14), quarter_duration),        # B5 quarter
        (note_freq(14), quarter_duration),        # B5 quarter

        # Ninth measure - New section
        (note_freq(14), quarter_duration),        # B5 quarter
        (note_freq(12), quarter_duration * 0.5),  # A5 eighth
        (note_freq(11), quarter_duration * 0.5),  # G#5 eighth
        (note_freq(9), quarter_duration),         # F#5 quarter

        # Tenth measure
        (note_freq(7), quarter_duration),         # E5 quarter
        (note_freq(9), quarter_duration),         # F#5 quarter
        (note_freq(11), quarter_duration),        # G#5 quarter

        # Eleventh measure
        (note_freq(12), quarter_duration),        # A5 quarter
        (note_freq(14), quarter_duration),        # B5 quarter
        (note_freq(16), quarter_duration),        # C#6 quarter

        # Twelfth measure
        (note_freq(14), quarter_duration * 1.5),  # B5 dotted quarter
        (note_freq(12), quarter_duration * 0.5),  # A5 eighth
        (note_freq(11), quarter_duration),        # G#5 quarter

        # Thirteenth measure
        (note_freq(9), quarter_duration),         # F#5 quarter
        (note_freq(11), quarter_duration),        # G#5 quarter
        (note_freq(12), quarter_duration),        # A5 quarter

        # Fourteenth measure
        (note_freq(14), quarter_duration),        # B5 quarter
        (note_freq(16), quarter_duration),        # C#6 quarter
        (note_freq(17), quarter_duration),        # D6 quarter

        # Fifteenth measure
        (note_freq(16), quarter_duration * 1.5),  # C#6 dotted quarter
        (note_freq(14), quarter_duration * 0.5),  # B5 eighth
        (note_freq(12), quarter_duration),        # A5 quarter

        # Sixteenth measure
        (note_freq(14), quarter_duration * 2),    # B5 half note
        (note_freq(14), quarter_duration),        # B5 quarter
    ]
    
    # Bass line (Piano LH) - Extended
    bass = [
        # First measure
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        
        # Second measure
        (note_freq(-5), quarter_duration),   # D4 quarter
        (note_freq(-5), quarter_duration),   # D4 quarter
        (note_freq(-5), quarter_duration),   # D4 quarter
        
        # Third measure
        (note_freq(-7), quarter_duration),   # C#4 quarter
        (note_freq(-7), quarter_duration),   # C#4 quarter
        (note_freq(-7), quarter_duration),   # C#4 quarter

        # Fourth measure
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter

        # Fifth measure
        (note_freq(-5), quarter_duration),   # D4 quarter
        (note_freq(-5), quarter_duration),   # D4 quarter
        (note_freq(-5), quarter_duration),   # D4 quarter

        # Sixth measure
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter

        # Seventh measure
        (note_freq(-3), quarter_duration),   # E4 quarter
        (note_freq(-3), quarter_duration),   # E4 quarter
        (note_freq(-3), quarter_duration),   # E4 quarter

        # Eighth measure
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter

        # Ninth measure - New section
        (note_freq(-3), quarter_duration),   # E4 quarter
        (note_freq(-3), quarter_duration),   # E4 quarter
        (note_freq(-3), quarter_duration),   # E4 quarter

        # Tenth measure
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter

        # Eleventh measure
        (note_freq(-5), quarter_duration),   # D4 quarter
        (note_freq(-5), quarter_duration),   # D4 quarter
        (note_freq(-5), quarter_duration),   # D4 quarter

        # Twelfth measure
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter

        # Thirteenth measure
        (note_freq(-7), quarter_duration),   # C#4 quarter
        (note_freq(-7), quarter_duration),   # C#4 quarter
        (note_freq(-7), quarter_duration),   # C#4 quarter

        # Fourteenth measure
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter

        # Fifteenth measure
        (note_freq(-5), quarter_duration),   # D4 quarter
        (note_freq(-5), quarter_duration),   # D4 quarter
        (note_freq(-5), quarter_duration),   # D4 quarter

        # Sixteenth measure
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
        (note_freq(-8), quarter_duration),   # B3 quarter
    ]
    
    samples = []
    bass_samples = []
    
    # Generate melody with attack and release
    for freq, duration in melody:
        note_samples = generate_square_wave(freq, duration, 0.3)
        # Add attack
        attack_samples = int(0.02 * sample_rate)  # 20ms attack
        for i in range(min(attack_samples, len(note_samples))):
            note_samples[i] *= (i / attack_samples)
        # Add release
        release_samples = int(0.03 * sample_rate)  # 30ms release
        for i in range(min(release_samples, len(note_samples))):
            note_samples[-(i+1)] *= (i / release_samples)
        samples.extend(note_samples)
    
    # Generate bass with longer attack and release
    for freq, duration in bass:
        note_samples = generate_square_wave(freq, duration, 0.2)
        # Add attack
        attack_samples = int(0.03 * sample_rate)  # 30ms attack
        for i in range(min(attack_samples, len(note_samples))):
            note_samples[i] *= (i / attack_samples)
        # Add release
        release_samples = int(0.04 * sample_rate)  # 40ms release
        for i in range(min(release_samples, len(note_samples))):
            note_samples[-(i+1)] *= (i / release_samples)
        bass_samples.extend(note_samples)
    
    # Ensure both arrays are the same length
    max_length = max(len(samples), len(bass_samples))
    samples.extend([0] * (max_length - len(samples)))
    bass_samples.extend([0] * (max_length - len(bass_samples)))
    
    # Mix melody and bass
    mixed_samples = [samples[i] + bass_samples[i] for i in range(max_length)]
    
    # Normalize
    max_amplitude = max(abs(min(mixed_samples)), abs(max(mixed_samples)))
    if max_amplitude > 0:
        scale_factor = 0.95 / max_amplitude
        mixed_samples = [s * scale_factor for s in mixed_samples]
    
    # Loop the sequence 2 times (it's twice as long now)
    return mixed_samples * 2

def main():
    """Generate all sound effects"""
    # Create sounds directory if it doesn't exist
    os.makedirs('assets/sounds', exist_ok=True)
    
    # Generate and save all sound effects
    save_wave_file('assets/sounds/move.wav', create_move_sound())
    save_wave_file('assets/sounds/rotate.wav', create_rotate_sound())
    save_wave_file('assets/sounds/drop.wav', create_drop_sound())
    save_wave_file('assets/sounds/clear.wav', create_clear_sound())
    save_wave_file('assets/sounds/tetris.wav', create_tetris_sound())
    save_wave_file('assets/sounds/game_over.wav', create_game_over_sound())
    save_wave_file('assets/sounds/background.wav', create_background_music())
    
    print("Sound effects generated successfully!")

if __name__ == '__main__':
    main() 
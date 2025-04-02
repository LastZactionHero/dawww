I'd like you to write a program that serves as the mixdown application for a DAW. Given a song file, it will mix the entire song down to a wave file. For now, it's just a simple sine wave synth, but this will grow to be more complicated. Consider the full spec documented in this folder.

I'd like you to write a program that:

- Reads a directory representing a song (sample in ./sample_song)
- Parses the JSON file representing the song in the format specified
- Saves internally in the app
- Generates a wav file for the song
- Saves the wav file to disk

Command should be:

mixdown ./sample_song ./sample_song.wav

Please write this in Rust.
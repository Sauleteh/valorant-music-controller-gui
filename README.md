# Valorant Music Controller (GUI)
Automatically pause/play and control the volume of your music depending on the state of the game you are in on Valorant. It should work on all music platforms, including YouTube, Spotify (browser and app), etc.
It currently supports three states on Valorant to determine the volume:
1. Not in game (No active game, choosing agent, map is loading)
2. In game - Preparing (Buy phase)
3. In game - Playing (Alive, playing the round)

This app uses the log file of the game to detect state changes in the game, so there aren't any restriction to use this program.

## Instructions
To get the app, just download the .exe <a href="https://github.com/Sauleteh/valorant-music-controller-gui/releases/latest">here</a> or build the source code with `cargo update` and `cargo build --release` (you will need rustup to compile).

![image](https://github.com/user-attachments/assets/a10d51b1-8b99-4544-86a2-166932fc1c6b)

1. Select the process that will change its volume (Firefox, Opera, Spotify...).
2. Adjust the volume to your liking for each state of the game.
3. Activate the program using the main button.
4. Start playing music and go to Valorant, good luck with the matches.

If you opened your music app after opening this program, use the "Update process list" button.

You can check if the program is working fine by using the "Simulate test" checkbox, this will make the main button to do a short simulation of a match. More information by clicking "how simulation works?" label.

If you forget how to use the app, there is a brief explanation on the "Help" button bar.

Note: Setting a volume to 0 on a state will pause the media player when this state is reached and will resume it when exiting this state.

## CLI version
Before i made this GUI version, i made a CLI version of this app, it's not very customizable as the GUI version but if you want to try it, you can check the CLI version <a href="https://github.com/Sauleteh/valorant-music-controller-cli">here</a>.

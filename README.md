# Beat Finder
This project is aimed for producers to help them categorize and find their beats faster so whenever they
have to show them to an artist, find one to sell, or simply find a project they're working on quickly, they
can do it with ease.
# Version Control
- 0.001: Initial commit, still a test project
- 0.002/0.003: Base to work
- 0.01: The player is able to detect all mp3's directly inside a folder (not inside sub-folders, though), and
play them whenever you click on one. It also launches a simple error window whenever an error occurs.
- 0.02: Right panel was added that shows the name of the current track being played, as well as providing buttons
for seeing in finder the current mp3 or its project file (should be named the same as the mp3 and be in the same folder).
Furthermore, it has a tag system in which you enter tags in the box (separated by newlines) and that tag is attached to
the mp3. The program also keeps track of all the tags
- 0.021: Added a safety mechanism so that the program only shows songs that are currently on the folder, and not the ones
that have been deleted. This results in deleted mp3s loosing its tags, and if it had tags that no other song has, those
also get removed from the global tag list.
### Note
Beat Finder uses EGUI under the hood and as such retains the licenses.
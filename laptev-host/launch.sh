#!/bin/sh

# this script should be executed on startup
# launches the server and motion capture script
# reboots after a day to prevent bugs

# i.e. copy the three lines bellow (uncomment after pasting them) in ~/.profile (or ~/.bash_login ~/.bash_profile for older systems)
#if [ -f ~/Laptev/launch.sh ]; then
#    if [ -z "$SSH_CONNECTION" ]; then
#        nohup ~/Laptev/launch.sh &
#    fi
#fi

sleep 5

laptev="/home/$USER/Laptev"
cd $laptev

if pgrep python; then pkill python; fi
if pgrep laptev-host; then pkill laptev-host; fi

nohup python "$laptev/motioncapture.py" > "$laptev/motioncapture.log" 2>&1 &
nohup "$laptev/laptev-host" > "$laptev/laptev-host.log" 2>&1 &

sleep 86400
reboot

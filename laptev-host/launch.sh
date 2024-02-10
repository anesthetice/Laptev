#!/bin/sh

# this script should be executed on startup
# launches the server and motion capture script
# reboots after a day to prevent bugs

# i.e. copy the three lines bellow (uncomment after pasting them) in ~/.profile (or ~/.bash_login ~/.bash_profile for older systems)
#if [ -f ~/Laptev/launch.sh ]; then
#    if [ -z "$SSH_CONNECTION" ]; then
#        if pgrep launch; then pkill launch; fi
#        if pgrep sleep; then pkill sleep; fi
#        if pgrep motioncapture; then pkill motioncapture; fi
#        if pgrep laptev-host; then pkill laptev-host; fi
#        nohup ~/Laptev/launch.sh > "/home/$USER/launch.log" 2>&1 &
#    fi
#fi

sleep 5

laptev="/home/$USER/Laptev"
cd $laptev

nohup "$laptev/motioncapture.py" > "$laptev/motioncapture.log" 2>&1 &
nohup "$laptev/laptev-host" > "$laptev/laptev-host.log" 2>&1 &

sleep 86400
reboot

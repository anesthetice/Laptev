#!/bin/sh

# this script should be executed by .bashrc
# launches the server and motion capture script
# reboots after a day to prevent ugly bugs

laptev="/home/$USER/Laptev"

nohup python "$laptev/motioncapture.py" > "$laptev/motioncapture.log"
nohup "$laptev/laptev-host" > "$laptev/laptev-host.log"

sleep 86400
reboot
#!/bin/bash
a=$(zenity --title neocities-sync --username --password)
ns_username=$(echo $a | cut -d'|' -f1)
ns_password=$(echo $a | cut -d'|' -f2)
ns_path=$(zenity --title "neocities-sync: Select directory to sync" --file-selection --directory)
if zenity --title "neocities-sync" --question --text "Do you want to ignore disallowed file types? (Say yes if you are NOT a neocities supporter)"; then
    bunx github:aspizu/neocities-sync --username $ns_username --password $ns_password --path $ns_path --ignore-disallowed-file-types &> /tmp/neocities-sync.log
else
    bunx github:aspizu/neocities-sync --username $ns_username --password $ns_password --path $ns_path &> /tmp/neocities-sync.log
fi
if [ $? -eq 0 ]; then
    zenity --title "neocities-sync" --info --text "Sync successful"
else
    zenity --title "neocities-sync" --error --text "Sync failed"
    xdg-open /tmp/neocities-sync.log
fi

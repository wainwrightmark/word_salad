for x in 70 76 120 150 152 192 310 512 1024 1080; do inkscape --export-type=png -o icon${x}.png -w ${x} icon.svg ; done
for x in 1024; do inkscape --export-type=png -o icon-white-circle${x}.png -w ${x} icon-white-circle.svg ; done
for x in 1024 ; do inkscape --export-type=png -o logo${x}.png -w ${x} logo.svg ; done
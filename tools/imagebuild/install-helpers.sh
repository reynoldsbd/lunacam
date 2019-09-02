#!/bin/false
# This file should only be dot-sourced

# Helpers for installing LunaCam

# Enumerates the specified directory ($1) and installs its contents to the current system,
# preserving directory structure.
#
# By default, contents are installed to the filesystem root. For instance, $1/etc/foo is installed
# as /etc/foo. An alternate destination may be specified with $2.
function install_dir {
    files=$(cd $1 && find -type f | cut -c 3-)
    for file in $files
    do
        echo "--> installing $2/$file"
        install -D $1/$file $2/$file
    done
}

# Common utilities used in LunaCam's pi-gen stages

function install_dir() {
    src=$1
    dst=$2
    mkdir -p $dst
    rsync -r $src $dst
    find $dst -type d -exec chmod 755 {} +
    find $dst -type f -exec chmod 644 {} +
}

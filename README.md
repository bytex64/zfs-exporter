# zfs-exporter

`zfs-exporter` is a prometheus exporter for ZFS space usage statistics.

## Status

This is a hack I threw together to make my life easier. It will probably
break with the next FreeBSD release. I might not maintain it. I guess in
theory this might work on Linux with ZFS patches but I haven't tried it.

## Usage

Compile the release version with `cargo build --release`. `make install`
will install the binary and rc file to /usr/local. Enable it with `sysrc
zfs_exporter_enable=YES`, and start it with `service zfs-exporter
start`. See the rc file for configurables.

## Metrics

Currently it only exports two metrics, `zfs_dataset_space_used` and
`zfs_dataset_space_available`. Each one is labeled with `pool`, which is
the pool the dataset belongs to; and `dataset`, which is the dataset
name.

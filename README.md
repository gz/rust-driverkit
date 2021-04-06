[![Build Status](https://travis-ci.org/gz/rust-driverkit.svg?branch=master)](https://travis-ci.org/gz/rust-driverkit)

# Driverkit

Framework for writing and simplifying testing of device drivers. This is work in progress.

## Usage

Using the DevMem type on Linux will require Hugepages:

```bash
echo 100 >/proc/sys/vm/nr_hugepages_mempolicy
echo 4 > /sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages_mempolicy
```

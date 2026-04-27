# SPDX-License-Identifier: GPL-2.0-or-later
#
# Out-of-tree kbuild Makefile for razer-blade-hwmon.
#
# Build against the running kernel:
#   make
# Build against an explicit kernel tree:
#   make KDIR=/lib/modules/$(uname -r)/build
# Install:
#   sudo make install && sudo depmod -a
# Clean:
#   make clean

obj-m := razer-blade-hwmon.o

KDIR  ?= /lib/modules/$(shell uname -r)/build
PWD   := $(shell pwd)

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

modules_install install:
	$(MAKE) -C $(KDIR) M=$(PWD) modules_install

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

.PHONY: all modules_install install clean

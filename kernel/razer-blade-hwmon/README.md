# razer-blade-hwmon

Linux kernel module that exposes Razer Blade laptop fan readings and
fan-mode control under the standard `hwmon` sysfs interface, so tools
like `sensors`, `fancontrol`, `psensor`, Home Assistant's
`systemmonitor`, etc. work without a userspace daemon.

## Why a kernel module?

Razer Blade laptops do **not** have a standard SIO/EC chip on the LPC
bus (no IT87xx, no NCT6xxx). The "EC" is a keyboard-controller MCU
(ENE/ITE family) running Razer firmware that enumerates as a USB HID
device under VID `0x1532`. Fan speeds, fan mode and performance modes
are driven through a 90-byte vendor-specific HID Feature Report — the
"Razer Report" — which is also what OpenRazer and the rest of this
repository (`crates/razer-core/src/packet.rs`,
`crates/razer-core/src/commands.rs`) speak.

That's why `sensors-detect` finds nothing useful and why a generic
hwmon chip driver isn't applicable. This module binds as a HID driver,
talks the Razer Report protocol from kernel space, and registers an
hwmon device on top.

## Supported devices

PIDs are kept in sync with `crates/razer-core/src/detect.rs`:

| PID    | Model                          |
|--------|--------------------------------|
| 0x028A | Razer Blade 15 (2022)          |
| 0x029D | Razer Blade 14 (2023)          |
| 0x029F | Razer Blade 16 (2023)          |
| 0x02B7 | Razer Blade 16 (2024)          |
| 0x02C6 | Razer Blade 16 (2025)          |
| 0x02E0 | Razer Blade 16 (2026)          |

If you have another Blade model, add its PID to the
`razer_hwmon_devices[]` table in `razer-blade-hwmon.c` (and to
`detect.rs` for the userspace tools).

## Exposed sysfs nodes

Under `/sys/class/hwmon/hwmonN/` (chip name `razerblade`):

| Attribute       | Mode | Meaning                                    |
|-----------------|------|--------------------------------------------|
| `fan1_input`    | r    | Measured CPU fan RPM (cmd `0x0D88`)        |
| `fan2_input`    | r    | Measured GPU fan RPM (cmd `0x0D88`)        |
| `fan1_label`    | r    | "CPU Fan"                                  |
| `fan2_label`    | r    | "GPU Fan"                                  |
| `fan1_target`   | rw   | Target RPM 0–5500 (cmd `0x0D01` / `0x0D81`)|
| `fan2_target`   | rw   | Target RPM (the EC sets both zones together; writing either updates both) |
| `pwm1_enable`   | rw   | `1` = manual, `2` = auto (cmd `0x0D02`/`0x0D82`) |

Notes:

- Writing `pwm1_enable=1` switches the fans to manual mode while keeping
  the current performance mode. Use `fan*_target` to set RPM.
- Writing `pwm1_enable=2` returns control to the EC's automatic curve.
- The EC quantises target RPM to multiples of 100.

## Build

Requires the matching kernel headers (Debian/Ubuntu:
`linux-headers-$(uname -r)`; Fedora: `kernel-devel`; Arch:
`linux-headers`).

```bash
cd kernel/razer-blade-hwmon
make
sudo insmod ./razer-blade-hwmon.ko
```

Verify:

```bash
sudo dmesg | grep razer-blade-hwmon
sensors razerblade-*
```

Permanent install:

```bash
sudo make install
sudo depmod -a
sudo modprobe razer-blade-hwmon
```

### DKMS (recommended for kernel upgrades)

```bash
sudo cp -r . /usr/src/razer-blade-hwmon-0.1.0
sudo dkms add    -m razer-blade-hwmon -v 0.1.0
sudo dkms build  -m razer-blade-hwmon -v 0.1.0
sudo dkms install -m razer-blade-hwmon -v 0.1.0
```

## Coexistence with userspace tools

The driver only binds the HID interface and uses `HID_CONNECT_HIDRAW`,
so OpenRazer, this repository's `rhelper-cli` / `razer-gui`, and any
other Razer Report consumer keep working in parallel. All HID
transactions inside the module are serialised by an internal mutex,
but kernel-side and userspace-side traffic are not — avoid hammering
the EC from both sides simultaneously.

## License

GPL-2.0-or-later. See SPDX header in `razer-blade-hwmon.c`.

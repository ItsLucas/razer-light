// SPDX-License-Identifier: GPL-2.0-or-later
/*
 * razer-blade-hwmon - hwmon driver for Razer Blade laptop EC
 *
 * The "EC" on Razer Blade laptops is not a standard SIO/EC chip on the LPC
 * bus. It is a keyboard-controller MCU (ENE/ITE family) running Razer
 * firmware and enumerated as a USB HID device under VID 0x1532. Fan
 * speeds, fan mode and performance modes are exchanged via a 90-byte
 * vendor-specific HID Feature Report (the so-called "Razer Report",
 * matching the layout used by OpenRazer and by this repository's
 * `crates/razer-core/src/packet.rs`).
 *
 * This driver binds to known Razer Blade PIDs as a HID driver, talks to
 * the EC over Feature Reports, and exposes the readings under the
 * standard hwmon sysfs interface so that `sensors`, `fancontrol`,
 * `psensor`, etc. can use them out of the box.
 *
 * Exposed channels:
 *   fan1_input, fan2_input   - measured RPM   (cmd 0x0D88)
 *   fan1_target, fan2_target - target RPM     (cmd 0x0D01 / 0x0D81)
 *   pwm1_enable              - 1=manual, 2=auto  (cmd 0x0D02 / 0x0D82)
 *
 * Copyright (C) 2026 razer-light contributors
 */

#include <linux/delay.h>
#include <linux/hid.h>
#include <linux/hwmon.h>
#include <linux/hwmon-sysfs.h>
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/mutex.h>
#include <linux/random.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/types.h>

#define RAZER_VID			0x1532

/* Razer Report wire format: 90 bytes payload + 1 byte HID report ID. */
#define RAZER_REPORT_LEN		90
#define RAZER_BUF_LEN			(RAZER_REPORT_LEN + 1)
#define RAZER_REPORT_ID			0x00

/* Status byte values (subset). */
#define RAZER_STATUS_NEW		0x00
#define RAZER_STATUS_BUSY		0x01
#define RAZER_STATUS_OK			0x02
#define RAZER_STATUS_FAIL		0x03
#define RAZER_STATUS_TIMEOUT		0x04
#define RAZER_STATUS_NOT_SUPPORTED	0x05

/* Commands used by this driver. */
#define CMD_GET_FAN_ACTUAL_RPM		0x0D88	/* args: 0x00, zone, 0x00 */
#define CMD_SET_FAN_RPM			0x0D01	/* args: 0x00, zone, rpm/100 */
#define CMD_GET_FAN_TARGET_RPM		0x0D81	/* args: 0x00, zone, 0x00 */
#define CMD_SET_PERF_MODE		0x0D02	/* args: 0x01, zone, perf, fan_mode */
#define CMD_GET_PERF_MODE		0x0D82	/* args: 0x00, 0x01, 0x00, 0x00 */

#define RAZER_FAN_ZONE1			0x01
#define RAZER_FAN_ZONE2			0x02

#define RAZER_FAN_MODE_AUTO		0x00
#define RAZER_FAN_MODE_MANUAL		0x01

/* Inter-transaction delays mirror the userspace implementation. */
#define RAZER_TX_DELAY_MS		1
#define RAZER_RX_DELAY_MS		2
#define RAZER_RETRY_DELAY_MS		25
#define RAZER_MAX_RETRIES		5

/* Maximum settable fan RPM (matches userspace bounds). */
#define RAZER_FAN_RPM_MAX		5500U

struct razer_report {
	u8	status;
	u8	transaction_id;
	__be16	remaining_packets;
	u8	protocol_type;
	u8	data_size;
	u8	command_class;
	u8	command_id;
	u8	args[80];
	u8	crc;
	u8	reserved;
} __packed;

static_assert(sizeof(struct razer_report) == RAZER_REPORT_LEN,
	      "razer_report must be exactly 90 bytes on the wire");

struct razer_hwmon {
	struct hid_device	*hdev;
	struct device		*hwmon_dev;
	struct mutex		io_lock;	/* serialises HID transactions */
	void			*xfer_buf;	/* RAZER_BUF_LEN, DMA-safe */
};

/* ---------- Razer Report helpers ---------- */

static u8 razer_report_crc(const struct razer_report *r)
{
	const u8 *p = (const u8 *)r;
	u8 crc = 0;
	size_t i;

	/* XOR over bytes [2..88) -- everything between transaction_id and crc. */
	for (i = 2; i < 88; i++)
		crc ^= p[i];
	return crc;
}

static void razer_report_init(struct razer_report *r, u16 command,
			      const u8 *args, size_t args_len)
{
	memset(r, 0, sizeof(*r));
	r->status = RAZER_STATUS_NEW;
	r->transaction_id = (u8)get_random_u32();
	r->remaining_packets = 0;
	r->protocol_type = 0x00;
	r->data_size = (u8)min_t(size_t, args_len, sizeof(r->args));
	r->command_class = (command >> 8) & 0xff;
	r->command_id = command & 0xff;
	if (args && args_len)
		memcpy(r->args, args, r->data_size);
	r->crc = razer_report_crc(r);
}

/* ---------- HID Feature Report transport ---------- */

/*
 * Send `req` and read back the response into `resp`.
 * Caller must hold rh->io_lock.
 */
static int razer_xfer_locked(struct razer_hwmon *rh,
			     const struct razer_report *req,
			     struct razer_report *resp)
{
	u8 *buf = rh->xfer_buf;
	int ret, attempt;

	for (attempt = 0; attempt < RAZER_MAX_RETRIES; attempt++) {
		msleep(RAZER_TX_DELAY_MS);

		buf[0] = RAZER_REPORT_ID;
		memcpy(buf + 1, req, RAZER_REPORT_LEN);

		ret = hid_hw_raw_request(rh->hdev, RAZER_REPORT_ID,
					 buf, RAZER_BUF_LEN,
					 HID_FEATURE_REPORT,
					 HID_REQ_SET_REPORT);
		if (ret < 0) {
			hid_dbg(rh->hdev,
				"SET_REPORT failed (attempt %d): %d\n",
				attempt, ret);
			goto retry;
		}

		msleep(RAZER_RX_DELAY_MS);

		memset(buf, 0, RAZER_BUF_LEN);
		buf[0] = RAZER_REPORT_ID;

		ret = hid_hw_raw_request(rh->hdev, RAZER_REPORT_ID,
					 buf, RAZER_BUF_LEN,
					 HID_FEATURE_REPORT,
					 HID_REQ_GET_REPORT);
		if (ret < 0) {
			hid_dbg(rh->hdev,
				"GET_REPORT failed (attempt %d): %d\n",
				attempt, ret);
			goto retry;
		}
		if (ret < RAZER_BUF_LEN) {
			hid_dbg(rh->hdev,
				"short GET_REPORT: %d < %d\n",
				ret, RAZER_BUF_LEN);
			ret = -EIO;
			goto retry;
		}

		memcpy(resp, buf + 1, RAZER_REPORT_LEN);

		if (resp->command_class != req->command_class ||
		    resp->command_id != req->command_id ||
		    resp->transaction_id != req->transaction_id) {
			hid_dbg(rh->hdev,
				"response mismatch: cmd=0x%02x%02x id=0x%02x (want 0x%02x%02x id=0x%02x)\n",
				resp->command_class, resp->command_id,
				resp->transaction_id,
				req->command_class, req->command_id,
				req->transaction_id);
			ret = -EBADMSG;
			goto retry;
		}

		switch (resp->status) {
		case RAZER_STATUS_OK:
			return 0;
		case RAZER_STATUS_NOT_SUPPORTED:
			return -EOPNOTSUPP;
		case RAZER_STATUS_BUSY:
		case RAZER_STATUS_TIMEOUT:
			ret = -EAGAIN;
			break;
		case RAZER_STATUS_FAIL:
		default:
			ret = -EIO;
			break;
		}

retry:
		if (ret == -EOPNOTSUPP)
			return ret;
		msleep(RAZER_RETRY_DELAY_MS);
	}

	return ret ?: -EIO;
}

static int razer_xfer(struct razer_hwmon *rh,
		      const struct razer_report *req,
		      struct razer_report *resp)
{
	int ret;

	mutex_lock(&rh->io_lock);
	ret = razer_xfer_locked(rh, req, resp);
	mutex_unlock(&rh->io_lock);
	return ret;
}

/* ---------- Razer command wrappers ---------- */

static int razer_get_fan_actual_rpm(struct razer_hwmon *rh, u8 zone, u16 *rpm)
{
	struct razer_report req, resp;
	u8 args[3] = { 0x00, zone, 0x00 };
	int ret;

	razer_report_init(&req, CMD_GET_FAN_ACTUAL_RPM, args, sizeof(args));
	ret = razer_xfer(rh, &req, &resp);
	if (ret)
		return ret;

	*rpm = (u16)resp.args[2] * 100;
	return 0;
}

static int razer_get_fan_target_rpm(struct razer_hwmon *rh, u8 zone, u16 *rpm)
{
	struct razer_report req, resp;
	u8 args[3] = { 0x00, zone, 0x00 };
	int ret;

	razer_report_init(&req, CMD_GET_FAN_TARGET_RPM, args, sizeof(args));
	ret = razer_xfer(rh, &req, &resp);
	if (ret)
		return ret;

	*rpm = (u16)resp.args[2] * 100;
	return 0;
}

static int razer_set_fan_rpm(struct razer_hwmon *rh, u16 rpm)
{
	struct razer_report req, resp;
	u8 args[3];
	u8 zone;
	int ret;

	if (rpm > RAZER_FAN_RPM_MAX)
		return -EINVAL;

	args[0] = 0x00;
	args[2] = (u8)(rpm / 100);

	for (zone = RAZER_FAN_ZONE1; zone <= RAZER_FAN_ZONE2; zone++) {
		args[1] = zone;
		razer_report_init(&req, CMD_SET_FAN_RPM, args, sizeof(args));
		ret = razer_xfer(rh, &req, &resp);
		if (ret)
			return ret;
	}
	return 0;
}

/* perf_mode: out (may be NULL), fan_mode: out (may be NULL) */
static int razer_get_perf_mode(struct razer_hwmon *rh,
			       u8 *perf_mode, u8 *fan_mode)
{
	struct razer_report req, resp;
	u8 args[4] = { 0x00, 0x01, 0x00, 0x00 };
	int ret;

	razer_report_init(&req, CMD_GET_PERF_MODE, args, sizeof(args));
	ret = razer_xfer(rh, &req, &resp);
	if (ret)
		return ret;

	if (perf_mode)
		*perf_mode = resp.args[2];
	if (fan_mode)
		*fan_mode = resp.args[3];
	return 0;
}

static int razer_set_fan_mode(struct razer_hwmon *rh, u8 fan_mode)
{
	struct razer_report req, resp;
	u8 perf_mode = 0;
	u8 args[4];
	u8 zone;
	int ret;

	ret = razer_get_perf_mode(rh, &perf_mode, NULL);
	if (ret)
		return ret;

	args[0] = 0x01;
	args[2] = perf_mode;
	args[3] = fan_mode;

	for (zone = RAZER_FAN_ZONE1; zone <= RAZER_FAN_ZONE2; zone++) {
		args[1] = zone;
		razer_report_init(&req, CMD_SET_PERF_MODE, args, sizeof(args));
		ret = razer_xfer(rh, &req, &resp);
		if (ret)
			return ret;
	}
	return 0;
}

/* ---------- hwmon callbacks ---------- */

static umode_t razer_hwmon_is_visible(const void *drvdata,
				      enum hwmon_sensor_types type,
				      u32 attr, int channel)
{
	switch (type) {
	case hwmon_fan:
		switch (attr) {
		case hwmon_fan_input:
			return 0444;
		case hwmon_fan_target:
			return 0644;
		default:
			return 0;
		}
	case hwmon_pwm:
		switch (attr) {
		case hwmon_pwm_enable:
			return 0644;
		default:
			return 0;
		}
	default:
		return 0;
	}
}

static const char * const razer_fan_labels[] = {
	"CPU Fan",
	"GPU Fan",
};

static int razer_hwmon_read_string(struct device *dev,
				   enum hwmon_sensor_types type,
				   u32 attr, int channel,
				   const char **str)
{
	if (type == hwmon_fan && attr == hwmon_fan_label) {
		if (channel < 0 || channel >= ARRAY_SIZE(razer_fan_labels))
			return -EINVAL;
		*str = razer_fan_labels[channel];
		return 0;
	}
	return -EOPNOTSUPP;
}

static int razer_hwmon_read(struct device *dev,
			    enum hwmon_sensor_types type,
			    u32 attr, int channel, long *val)
{
	struct razer_hwmon *rh = dev_get_drvdata(dev);
	u16 rpm = 0;
	u8 fan_mode = 0;
	int ret;
	u8 zone;

	if (channel == 0)
		zone = RAZER_FAN_ZONE1;
	else if (channel == 1)
		zone = RAZER_FAN_ZONE2;
	else
		return -EINVAL;

	switch (type) {
	case hwmon_fan:
		switch (attr) {
		case hwmon_fan_input:
			ret = razer_get_fan_actual_rpm(rh, zone, &rpm);
			if (ret)
				return ret;
			*val = rpm;
			return 0;
		case hwmon_fan_target:
			ret = razer_get_fan_target_rpm(rh, zone, &rpm);
			if (ret)
				return ret;
			*val = rpm;
			return 0;
		default:
			return -EOPNOTSUPP;
		}
	case hwmon_pwm:
		switch (attr) {
		case hwmon_pwm_enable:
			ret = razer_get_perf_mode(rh, NULL, &fan_mode);
			if (ret)
				return ret;
			/* hwmon convention: 1=manual, 2=auto */
			*val = (fan_mode == RAZER_FAN_MODE_MANUAL) ? 1 : 2;
			return 0;
		default:
			return -EOPNOTSUPP;
		}
	default:
		return -EOPNOTSUPP;
	}
}

static int razer_hwmon_write(struct device *dev,
			     enum hwmon_sensor_types type,
			     u32 attr, int channel, long val)
{
	struct razer_hwmon *rh = dev_get_drvdata(dev);

	switch (type) {
	case hwmon_fan:
		if (attr != hwmon_fan_target)
			return -EOPNOTSUPP;
		if (val < 0 || val > RAZER_FAN_RPM_MAX)
			return -EINVAL;
		/* The EC sets both zones in lock-step; channel is ignored. */
		return razer_set_fan_rpm(rh, (u16)val);
	case hwmon_pwm:
		if (attr != hwmon_pwm_enable)
			return -EOPNOTSUPP;
		switch (val) {
		case 1:
			return razer_set_fan_mode(rh, RAZER_FAN_MODE_MANUAL);
		case 2:
			return razer_set_fan_mode(rh, RAZER_FAN_MODE_AUTO);
		default:
			return -EINVAL;
		}
	default:
		return -EOPNOTSUPP;
	}
}

static const struct hwmon_channel_info *razer_hwmon_info[] = {
	HWMON_CHANNEL_INFO(fan,
			   HWMON_F_INPUT | HWMON_F_TARGET | HWMON_F_LABEL,
			   HWMON_F_INPUT | HWMON_F_TARGET | HWMON_F_LABEL),
	HWMON_CHANNEL_INFO(pwm,
			   HWMON_PWM_ENABLE),
	NULL
};

static const struct hwmon_ops razer_hwmon_ops = {
	.is_visible	= razer_hwmon_is_visible,
	.read		= razer_hwmon_read,
	.read_string	= razer_hwmon_read_string,
	.write		= razer_hwmon_write,
};

static const struct hwmon_chip_info razer_hwmon_chip_info = {
	.ops	= &razer_hwmon_ops,
	.info	= razer_hwmon_info,
};

/* ---------- HID driver glue ---------- */

/*
 * Razer laptops expose multiple HID interfaces. Only one of them
 * accepts the vendor Feature Report. Probe by issuing a harmless
 * GET_PERF_MODE; if the EC answers, this is the right interface and
 * we register an hwmon device on it. Otherwise we still bind (so
 * other interfaces don't get rebound by accident) but skip hwmon.
 */
static int razer_probe_ec(struct razer_hwmon *rh)
{
	u8 perf = 0, fan = 0;

	return razer_get_perf_mode(rh, &perf, &fan);
}

static int razer_hwmon_probe(struct hid_device *hdev,
			     const struct hid_device_id *id)
{
	struct razer_hwmon *rh;
	int ret;

	rh = devm_kzalloc(&hdev->dev, sizeof(*rh), GFP_KERNEL);
	if (!rh)
		return -ENOMEM;

	rh->hdev = hdev;
	mutex_init(&rh->io_lock);

	rh->xfer_buf = devm_kzalloc(&hdev->dev, RAZER_BUF_LEN, GFP_KERNEL);
	if (!rh->xfer_buf)
		return -ENOMEM;

	hid_set_drvdata(hdev, rh);

	ret = hid_parse(hdev);
	if (ret) {
		hid_err(hdev, "hid_parse failed: %d\n", ret);
		return ret;
	}

	/*
	 * We only need the control channel for Feature Reports; no input
	 * events are consumed.  HID_CONNECT_HIDRAW keeps existing userspace
	 * (OpenRazer / this repo's CLI) working in parallel.
	 */
	ret = hid_hw_start(hdev, HID_CONNECT_HIDRAW);
	if (ret) {
		hid_err(hdev, "hid_hw_start failed: %d\n", ret);
		return ret;
	}

	ret = hid_hw_open(hdev);
	if (ret) {
		hid_err(hdev, "hid_hw_open failed: %d\n", ret);
		goto err_stop;
	}

	ret = razer_probe_ec(rh);
	if (ret) {
		hid_dbg(hdev,
			"EC probe failed on this interface (%d); skipping hwmon\n",
			ret);
		/* Not fatal: just don't expose hwmon on this interface. */
		return 0;
	}

	rh->hwmon_dev = devm_hwmon_device_register_with_info(
		&hdev->dev, "razerblade", rh,
		&razer_hwmon_chip_info, NULL);
	if (IS_ERR(rh->hwmon_dev)) {
		ret = PTR_ERR(rh->hwmon_dev);
		hid_err(hdev, "hwmon_device_register failed: %d\n", ret);
		goto err_close;
	}

	hid_info(hdev, "razer-blade-hwmon registered (PID 0x%04x)\n",
		 hdev->product);
	return 0;

err_close:
	hid_hw_close(hdev);
err_stop:
	hid_hw_stop(hdev);
	return ret;
}

static void razer_hwmon_remove(struct hid_device *hdev)
{
	hid_hw_close(hdev);
	hid_hw_stop(hdev);
}

/*
 * Supported PIDs.  Keep in sync with crates/razer-core/src/detect.rs.
 */
static const struct hid_device_id razer_hwmon_devices[] = {
	{ HID_USB_DEVICE(RAZER_VID, 0x028A) }, /* Blade 15 (2022)   */
	{ HID_USB_DEVICE(RAZER_VID, 0x029D) }, /* Blade 14 (2023)   */
	{ HID_USB_DEVICE(RAZER_VID, 0x029F) }, /* Blade 16 (2023)   */
	{ HID_USB_DEVICE(RAZER_VID, 0x02B7) }, /* Blade 16 (2024)   */
	{ HID_USB_DEVICE(RAZER_VID, 0x02C6) }, /* Blade 16 (2025)   */
	{ HID_USB_DEVICE(RAZER_VID, 0x02E0) }, /* Blade 16 (2026)   */
	{ }
};
MODULE_DEVICE_TABLE(hid, razer_hwmon_devices);

static struct hid_driver razer_hwmon_driver = {
	.name		= "razer-blade-hwmon",
	.id_table	= razer_hwmon_devices,
	.probe		= razer_hwmon_probe,
	.remove		= razer_hwmon_remove,
};
module_hid_driver(razer_hwmon_driver);

MODULE_AUTHOR("razer-light contributors");
MODULE_DESCRIPTION("hwmon driver for Razer Blade laptop EC (HID Feature Reports)");
MODULE_LICENSE("GPL");
MODULE_VERSION("0.1.0");

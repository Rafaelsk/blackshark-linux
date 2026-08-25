use anyhow::{bail, Context, Result};
use hidapi::{HidApi, HidDevice};
use tracing::info;

use blackshark_protocol::{Report, ResponseStatus, REPORT_LEN};

const VID: u16 = 0x1532;

/// Wired USB-audio identity for the BlackShark V3 Pro Xbox edition.
///
/// Its interface 5 advertises the same report layout as the dongle, but does
/// not answer the proprietary command protocol. It is presence-only here and
/// must not be returned by [`open`].
const WIRED_PID: u16 = 0x0a4e;

/// Product IDs for the dongle, covering both the PC and Xbox editions —
/// they share the same proprietary HID protocol.
const PIDS: &[u16] = &[
    0x0577, // BlackShark V3 Pro
    0x0a55, // BlackShark V3 Pro for Xbox
];

/// Whether the wired USB-audio transport is currently attached.
///
/// This intentionally enumerates without opening the HID interface. The wired
/// device is useful as a transport-presence signal, not as a command channel.
pub fn wired_present() -> Result<bool> {
    let api = HidApi::new().context("failed to initialise hidapi")?;

    let present = api.device_list().any(|info| {
        info.vendor_id() == VID && info.product_id() == WIRED_PID && info.interface_number() == 5
    });

    Ok(present)
}

/// Open the BlackShark V3 Pro HID device.
///
/// Must open interface 5 specifically — the dongle exposes multiple HID interfaces
/// and api.open(VID, PID) picks the first enumerated, which varies across systems.
/// Interface 5 is the proprietary control interface (interrupt IN, endpoint 0x84).
pub fn open() -> Result<HidDevice> {
    let api = HidApi::new().context("failed to initialise hidapi")?;

    let mut target = None;
    for info in api.device_list() {
        if info.vendor_id() == VID && PIDS.contains(&info.product_id()) {
            let path = info.path().to_string_lossy();
            info!(
                interface = info.interface_number(),
                path = %path,
                "found BlackShark hidraw interface"
            );
            if info.interface_number() == 5 {
                target = Some(info.clone());
            }
        }
    }

    match target {
        None => bail!("BlackShark V3 Pro (or Xbox edition) not found — is the dongle plugged in and do you have udev permission?"),
        Some(info) => {
            let path = info.path().to_string_lossy().into_owned();
            let dev = info
                .open_device(&api)
                .context("found BlackShark V3 Pro but failed to open control interface — check udev permissions")?;
            info!(path = %path, "opened BlackShark control interface");
            Ok(dev)
        }
    }
}

/// Send a report and return Ok if ANY 64-byte response arrives (regardless of status).
/// Used as a wireless link readiness probe — any response means the link is up.
pub fn send_probe(dev: &HidDevice, report: &Report) -> Result<()> {
    dev.write(report.as_bytes()).context("HID write failed")?;
    let mut buf = [0u8; REPORT_LEN];
    let n = dev
        .read_timeout(&mut buf, 2_000)
        .context("HID read failed")?;
    if n != REPORT_LEN {
        bail!("short read: expected {REPORT_LEN} bytes, got {n}");
    }
    Ok(())
}

/// Write a report without waiting for a response (fire-and-forget).
/// Used for init handshake commands where the side-effect of sending
/// matters but the response may not arrive or may be ignored.
pub fn send_no_wait(dev: &HidDevice, report: &Report) -> Result<()> {
    dev.write(report.as_bytes()).context("HID write failed")?;
    Ok(())
}

/// Send a report and read back the response.
///
/// Razer devices echo the command back with the status byte set.
pub fn send(dev: &HidDevice, report: &Report) -> Result<Report> {
    dev.write(report.as_bytes()).context("HID write failed")?;

    let expected = report.as_bytes();

    loop {
        let mut buf = [0u8; REPORT_LEN];
        let n = dev
            .read_timeout(&mut buf, 5_000)
            .context("HID read failed")?;

        if n != REPORT_LEN {
            bail!("short read: expected {REPORT_LEN} bytes, got {n}");
        }

        let response = Report::from_bytes(buf);

        if response.as_bytes()[2] != expected[2] || response.as_bytes()[10] != expected[10] {
            continue;
        }

        return match response.status() {
            ResponseStatus::Ok => Ok(response),
            other => bail!(
                "device returned error status: {other:?} (raw=0x{:02x})",
                response.as_bytes()[1]
            ),
        };
    }
}

pub fn read(dev: &HidDevice, timeout_ms: i32) -> Result<Option<Report>> {
    let mut buf = [0u8; REPORT_LEN];

    let n = dev
        .read_timeout(&mut buf, timeout_ms)
        .context("HID read failed")?;

    match n {
        0 => Ok(None),
        REPORT_LEN => Ok(Some(Report::from_bytes(buf))),
        other => bail!("short read: expected {REPORT_LEN} bytes, got {other}"),
    }
}

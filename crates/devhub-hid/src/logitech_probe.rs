//! Read-only HID++ capability discovery. Not registered as a settings driver.
//! Sends only ROOT getFeature and FEATURE_SET getters; no mode, LED, profile,
//! pairing, EEPROM or firmware commands are provided by this API.
use devhub_core::backend::{HubError, HubResult};
use hidapi::HidDevice;
use std::time::{Duration, Instant};

/// The physical long-report collections observed on the target Windows PC.
pub fn is_target(pid: u16, page: u16, usage: u16) -> bool {
    matches!(
        (pid, page, usage),
        (0xc095, 0xff00, 2) | (0xc339, 0xff43, 0x0602)
    )
}

pub struct Probe {
    device: HidDevice,
    software_id: u8,
}

impl Probe {
    pub fn new(device: HidDevice) -> Self {
        Self {
            device,
            software_id: 0,
        }
    }

    // Deliberately private: callers cannot send arbitrary commands.
    fn get(&mut self, feature: u8, function: u8, parameters: [u8; 3]) -> HubResult<Vec<u8>> {
        self.software_id = self.software_id % 15 + 1;
        let header = [0xff, feature, (function << 4) | self.software_id];
        let mut frame = [0u8; 20];
        frame[0] = 0x11;
        frame[1..4].copy_from_slice(&header);
        frame[4..7].copy_from_slice(&parameters);
        let written = self
            .device
            .write(&frame)
            .map_err(|e| HubError::Io(e.to_string()))?;
        if written != frame.len() {
            return Err(HubError::Protocol("HID++ short write".into()));
        }
        let deadline = Instant::now() + Duration::from_millis(750);
        while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
            let mut reply = [0u8; 64];
            let n = self
                .device
                .read_timeout(&mut reply, remaining.as_millis().max(1) as i32)
                .map_err(|e| HubError::Io(e.to_string()))?;
            if let Some(result) = parse_reply(&reply[..n], header) {
                return result;
            }
        }
        Err(HubError::Protocol("HID++ getter timed out".into()))
    }

    /// ROOT.getFeature: index, flags, version. Index zero means absent.
    pub fn feature(&mut self, id: u16) -> HubResult<Option<[u8; 3]>> {
        let [hi, lo] = id.to_be_bytes();
        let reply = self.get(0, 0, [hi, lo, 0])?;
        Ok((reply[0] != 0).then_some([reply[0], reply[1], reply[2]]))
    }

    /// Enumerate feature IDs without reading settings or changing modes.
    pub fn features(&mut self) -> HubResult<Vec<(u16, u8, u8)>> {
        let set = self
            .feature(1)?
            .ok_or(HubError::Unsupported("HID++ FEATURE_SET"))?[0];
        let count = self.get(set, 0, [0; 3])?[0];
        let mut out = Vec::new();
        for index in 1..=count {
            let reply = self.get(set, 1, [index, 0, 0])?;
            if reply.len() < 4 {
                return Err(HubError::Protocol(
                    "HID++ feature entry is truncated".into(),
                ));
            }
            out.push((u16::from_be_bytes([reply[0], reply[1]]), reply[2], reply[3]));
        }
        Ok(out)
    }
}

fn parse_reply(reply: &[u8], header: [u8; 3]) -> Option<HubResult<Vec<u8>>> {
    let expected = match reply.first()? {
        0x10 => 7,
        0x11 => 20,
        _ => return None,
    };
    if reply.len() != expected {
        return None;
    }
    if reply.get(1..4) == Some(header.as_slice()) {
        return Some(Ok(reply[4..].to_vec()));
    }
    if reply[1..5] == [header[0], 0xff, header[1], header[2]]
        || reply[1..5] == [header[0], 0x8f, header[1], header[2]]
    {
        return Some(Err(HubError::Protocol(format!(
            "HID++ error {:02x}",
            reply[5]
        ))));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_physical_vendor_collection() {
        assert!(is_target(0xc095, 0xff00, 2));
        assert!(is_target(0xc339, 0xff43, 0x0602));
        assert!(!is_target(0xc095, 0x59, 1)); // Windows virtual lighting device
        assert!(!is_target(0xc547, 0xff00, 2)); // receiver needs slot discovery
        assert!(!is_target(0xc339, 1, 6)); // ordinary keyboard input
    }
    #[test]
    fn rejects_truncated_and_unrelated_responses() {
        let header = [0xff, 4, 0x12];
        assert!(parse_reply(&[0x11, 0xff, 4, 0x12, 1], header).is_none());
        let mut reply = [0; 20];
        reply[..4].copy_from_slice(&[0x11, 0xff, 4, 0x13]);
        assert!(parse_reply(&reply, header).is_none());
        reply[3] = 0x12;
        assert!(parse_reply(&reply, header).unwrap().is_ok());
    }
    #[test]
    fn matches_error_to_request() {
        let header = [0xff, 4, 0x12];
        assert!(parse_reply(&[0x10, 0xff, 0xff, 4, 0x12, 2, 0], header)
            .unwrap()
            .is_err());
        assert!(parse_reply(&[0x10, 0xff, 0xff, 5, 0x12, 2, 0], header).is_none());
    }
}

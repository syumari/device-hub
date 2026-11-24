//! Default: enumerate only. --query: allowlisted HID++ feature discovery.
use devhub_hid::logitech_probe::{is_target, Probe};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a != "--query") {
        return Err("Use no arguments or --query".into());
    }
    let query = args.iter().any(|a| a == "--query");
    if query {
        #[cfg(windows)]
        {
            let processes = std::process::Command::new("tasklist")
                .args(["/FO", "CSV", "/NH"])
                .output()?;
            if !processes.status.success() {
                return Err("Cannot check G HUB process state".into());
            }
            let text = String::from_utf8_lossy(&processes.stdout).to_ascii_lowercase();
            if ["lghub_agent.exe", "lghub.exe", "lghub_system_tray.exe"]
                .iter()
                .any(|name| text.contains(name))
            {
                return Err(
                    "Exit G HUB before --query; enumeration without --query is safe while it runs"
                        .into(),
                );
            }
        }
    }
    let api = hidapi::HidApi::new()?;
    for d in api.device_list().filter(|d| d.vendor_id() == 0x046d) {
        println!(
            "{:04x}:{:04x} if={} page={:04x} usage={:04x} product={:?}",
            d.vendor_id(),
            d.product_id(),
            d.interface_number(),
            d.usage_page(),
            d.usage(),
            d.product_string()
        );
        if query && is_target(d.product_id(), d.usage_page(), d.usage()) {
            let mut probe = Probe::new(d.open_device(&api)?);
            for (id, flags, version) in probe.features()? {
                println!("  feature={id:04x} flags={flags:02x} version={version}");
            }
        }
    }
    Ok(())
}

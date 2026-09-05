//! Best-effort forensic / environment detection
use std::path::Path;

pub struct ForensicDetector;

impl ForensicDetector {
    pub fn is_emulator() -> bool {
        #[cfg(target_os = "android")]
        {
            let props = [
                "/dev/socket/qemud",
                "/dev/qemu_pipe",
                "/system/lib/libc_malloc_debug_qemu.so",
            ];
            if props.iter().any(|p| Path::new(p).exists()) { return true; }
        }
        false
    }

    pub fn is_rooted() -> bool {
        #[cfg(target_os = "android")]
        {
            let indicators = [
                "/system/bin/su",
                "/system/xbin/su",
                "/sbin/su",
                "/data/local/xbin/su",
                "/system/app/Superuser.apk",
                "/system/app/SuperSU.apk",
            ];
            return indicators.iter().any(|p| Path::new(p).exists());
        }
        #[cfg(not(target_os = "android"))]
        false
    }

    pub fn risk_level() -> u8 {
        let mut score = 0u8;
        if Self::is_emulator() { score += 2; }
        if Self::is_rooted() { score += 3; }
        score
    }
}

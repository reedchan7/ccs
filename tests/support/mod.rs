use std::path::{Path, PathBuf};

use tempfile::TempDir;

#[allow(dead_code)]
pub struct TestHome {
    temp: TempDir,
    bin: PathBuf,
}

#[allow(dead_code)]
impl TestHome {
    pub fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let bin = temp.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        Self { temp, bin }
    }

    pub fn path(&self) -> &Path {
        self.temp.path()
    }

    pub fn bin(&self) -> &Path {
        &self.bin
    }

    pub fn write_fake_claude(&self) {
        let path = self.bin.join("claude");
        std::fs::write(
            &path,
            "#!/usr/bin/env bash\nprintf 'CCS_ACTIVE_PROFILE=%s\\n' \"${CCS_ACTIVE_PROFILE:-}\"\nprintf 'CLAUDE_CONFIG_DIR=%s\\n' \"${CLAUDE_CONFIG_DIR:-}\"\nprintf 'ANTHROPIC_BASE_URL=%s\\n' \"${ANTHROPIC_BASE_URL:-}\"\nprintf 'ANTHROPIC_AUTH_TOKEN=%s\\n' \"${ANTHROPIC_AUTH_TOKEN:-}\"\nprintf 'Z_AI_API_KEY=%s\\n' \"${Z_AI_API_KEY:-}\"\nprintf 'Z_AI_MODE=%s\\n' \"${Z_AI_MODE:-}\"\nprintf 'ARGS='\nprintf '%s ' \"$@\"\nprintf '\\n'\n",
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}

fn main() {
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        use std::process::Command;

        let res_path = Path::new("assets/icons/app_icon.res");

        let rc_candidates = [
            r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\rc.exe",
            r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\rc.exe",
            r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\rc.exe",
            "rc.exe",
        ];

        for rc in rc_candidates {
            if Path::new(rc).exists() || rc == "rc.exe" {
                let status = Command::new(rc)
                    .args(&[
                        "/fo",
                        "assets/icons/app_icon.res",
                        "assets/icons/app_icon.rc",
                    ])
                    .status();
                if let Ok(s) = status {
                    if s.success() {
                        break;
                    }
                }
            }
        }

        if res_path.exists() {
            let abs_path = std::fs::canonicalize(res_path).unwrap();
            println!("cargo:rustc-link-arg={}", abs_path.display());
        }

        println!("cargo:rerun-if-changed=assets/icons/app_icon.rc");
        println!("cargo:rerun-if-changed=assets/icons/app_icon.ico");
    }
}

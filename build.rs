use std::{env, fs, path::PathBuf, process::Command};

fn resource_compiler() -> PathBuf {
    let root = PathBuf::from(
        env::var_os("ProgramFiles(x86)").unwrap_or_else(|| r"C:\Program Files (x86)".into()),
    )
    .join(r"Windows Kits\10\bin");
    let architecture = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "x64",
        Ok("aarch64") => "arm64",
        _ => "x86",
    };
    let mut candidates: Vec<_> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path().join(architecture).join("rc.exe"))
        .filter(|path| path.is_file())
        .collect();
    candidates.sort();
    candidates.pop().unwrap_or_else(|| PathBuf::from("rc.exe"))
}

fn main() {
    println!("cargo:rerun-if-changed=assets/branding/vandovolt.ico");
    println!("cargo:rerun-if-changed=Cargo.toml");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let version = env::var("CARGO_PKG_VERSION").expect("package version");
    let mut parts = version
        .split('.')
        .map(|part| part.parse::<u16>().unwrap_or(0));
    let numbers = (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        0,
    );
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("output directory"));
    let icon = manifest.join("assets/branding/vandovolt.ico");
    let icon_resource_path = icon.to_string_lossy().replace('\\', "/");
    let resource = output.join("vandovolt.rc");
    let compiled = output.join("vandovolt.res");
    let contents = format!(
        r#"1 ICON "{}"
1 VERSIONINFO
FILEVERSION {},{},{},{}
PRODUCTVERSION {},{},{},{}
FILEFLAGSMASK 0x3fL
FILEOS 0x40004L
FILETYPE 0x1L
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "vandonsc\0"
      VALUE "FileDescription", "Wireless Peripheral Battery Monitor\0"
      VALUE "FileVersion", "{}\0"
      VALUE "InternalName", "VandoVolt\0"
      VALUE "OriginalFilename", "VandoVolt.exe\0"
      VALUE "ProductName", "VandoVolt\0"
      VALUE "ProductVersion", "{}\0"
      VALUE "LegalCopyright", "Copyright (c) vandonsc\0"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x0409, 1200
  END
END
"#,
        icon_resource_path,
        numbers.0,
        numbers.1,
        numbers.2,
        numbers.3,
        numbers.0,
        numbers.1,
        numbers.2,
        numbers.3,
        version,
        version
    );
    fs::write(&resource, contents).expect("write Windows resource script");
    let status = Command::new(resource_compiler())
        .arg("/nologo")
        .arg(format!("/fo{}", compiled.display()))
        .arg(&resource)
        .status()
        .expect("run Windows resource compiler (rc.exe)");
    assert!(status.success(), "rc.exe failed");
    println!("cargo:rustc-link-arg-bin=VandoVolt={}", compiled.display());
}

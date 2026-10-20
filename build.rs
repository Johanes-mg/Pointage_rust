fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("Images/logo.ico");
        res.set("ProductName", "Pointage");
        res.set("FileDescription", "Application de pointage");
        res.set("CompanyName", "Johanes-mg");
        res.set("LegalCopyright", "Johanes-mg");
        res.set("FileVersion", "2.2.0");
        res.set("ProductVersion", "2.2.0");
        res.set("OriginalFilename", "pointage.exe");
        res.set("InternalName", "pointage");
        res.set_language(0x0409);
        match res.compile() {
            Ok(()) => {
                println!("build.rs: ressources Windows compilees (icone + version).");
            }
            Err(e) => {
                println!("cargo:warning=build.rs: echec compilation: {}", e);
            }
        }
    }

    println!("cargo:rerun-if-changed=Images/logo.ico");
    println!("cargo:rerun-if-changed=build.rs");
}

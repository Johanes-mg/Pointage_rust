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
        let _ = res.compile();
    }
}

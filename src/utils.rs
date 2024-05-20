pub mod browser {
    //! utils of open browser
    #[cfg(target_os = "macos")]
    pub fn open(url: &str) -> std::io::Result<()> {
        use std::process::Command;
        Command::new("open").arg(url).spawn()?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn open(url: &str) -> std::io::Result<()> {
        use std::process::Command;
        Command::new("cmd").args(&["/C", "start", url]).spawn()?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn open(url: &str) -> std::io::Result<()> {
        use std::process::Command;
        Command::new("xdg-open").arg(url).spawn()?;
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    pub fn open(_url: &str) -> std::io::Result<()> {
        println!("Invalid operator");
        Ok(())
    }
}

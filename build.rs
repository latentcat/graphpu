use std::io;
#[cfg(target_os = "windows")] use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(target_os = "windows")] {
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("resources/app_icon.ico")
            .compile()?;
    }
    Ok(())
}
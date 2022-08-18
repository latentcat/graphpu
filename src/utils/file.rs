use std::{path::PathBuf, process::Command};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use directories::UserDirs;
use crate::models::graphics_lib::BufferDimensions;
use crate::utils::message::{message_error, message_info};

pub fn pick_folder() -> Option<PathBuf> {
    let dir = desktop_dir_or_empty();

    rfd::FileDialog::new()
        .set_directory(dir)
        .pick_folder()
}

pub fn pick_csv() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text File", &["txt", "csv"])
        .pick_file()
}

pub fn path_to_string(path: &Option<PathBuf>) -> Option<String> {
    path.as_ref().map(|path| path.display().to_string())
}

pub fn system_open_directory(output_directory: &str) {
    if cfg!(windows) {
        Command::new("explorer")
            .arg(output_directory) // <- Specify the directory you'd like to open.
            .spawn()
            .unwrap();
    } else if cfg!(unix) {
        Command::new("open")
            .arg(output_directory) // <- Specify the directory you'd like to open.
            .spawn()
            .unwrap();
    }
}

pub fn desktop_dir_or_empty() -> String {
    let mut dir = String::from("");
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(desktop_dir) = user_dirs.desktop_dir() {
            dir = desktop_dir.display().to_string();
        }
    }
    dir
}



pub async fn create_png(
    png_output_path: String,
    device: &Arc<wgpu::Device>,
    output_buffer: wgpu::Buffer,
    buffer_dimensions: &BufferDimensions,
    submission_index: wgpu::SubmissionIndex,
) {
    // Note that we're not calling `.await` here.
    let buffer_slice = output_buffer.slice(..);
    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    //
    // We pass our submission index so we don't need to wait for any other possible submissions.
    device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
    // If a file system is available, write the buffer as a PNG
    let has_file_system_available = cfg!(not(target_arch = "wasm32"));
    if !has_file_system_available {
        return;
    }

    if let Some(Ok(())) = receiver.receive().await {
        let padded_buffer = buffer_slice.get_mapped_range();

        match File::create(&png_output_path) {
            Ok(file) => {
                let mut png_encoder = png::Encoder::new(
                    file,
                    buffer_dimensions.width as u32,
                    buffer_dimensions.height as u32,
                );
                png_encoder.set_depth(png::BitDepth::Eight);
                png_encoder.set_color(png::ColorType::Rgba);
                // png_encoder.set_source_gamma(png::ScaledFloat::new(1.0));
                // png_encoder.set_srgb(png::SrgbRenderingIntent::AbsoluteColorimetric);
                let mut png_writer = png_encoder
                    .write_header()
                    .unwrap()
                    .into_stream_writer_with_size(buffer_dimensions.unpadded_bytes_per_row)
                    .unwrap();

                // from the padded_buffer we write just the unpadded bytes into the image
                for chunk in padded_buffer.chunks(buffer_dimensions.padded_bytes_per_row) {
                    png_writer
                        .write_all(&chunk[..buffer_dimensions.unpadded_bytes_per_row])
                        .unwrap();
                }
                png_writer.finish().unwrap();

                // With the current interface, we have to make sure all mapped views are
                // dropped before we unmap the buffer.
                drop(padded_buffer);

                output_buffer.unmap();

                message_info("Output Succeeded", png_output_path.to_owned().as_str())
            },
            Err(err) => {
                message_error("create_png", &err.to_string());
            }
        }
    }
}